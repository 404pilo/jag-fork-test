//! C-cli: the stop of a search on Ctrl-C or on a closed pipe (CV-4, STORY-43,
//! DECISION-39).
//!
//! A stop sets the flag of `score`, takes the one right to write the output,
//! and ends jag with the default action of the signal, as grep ends. Until
//! the search flow or a stop takes the output, nobody writes it.

use std::fs::File;
use std::io::{self, Write};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::fs::FileTypeExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::io::Errno;
use signal_hook::consts::{SIGINT, SIGPIPE};
use signal_hook::iterator::Signals;
use signal_hook::low_level::emulate_default_handler;

use super::{Mode, error_document};
use crate::failure::Failure;
use crate::output::Summary;
use crate::output::progress::Progress;
use crate::scorer::ScoreError;

/// How often the watch thread asks the system about the pipe.
const WATCH_EVERY: Duration = Duration::from_millis(100);

/// Who owns the output (decision 14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    Nobody,
    Search,
    Stop,
}

/// The stop of a search on SIGINT or on a closed pipe (CV-4, DECISION-39).
pub struct Stop {
    /// Set first by a stop: no model call and no retry starts after it.
    flag: AtomicBool,
    owner: Mutex<Owner>,
    /// What jag counted, for the document of a stop in structured mode.
    counted: Mutex<Summary>,
    mode: Mode,
    progress: Arc<Progress>,
}

/// A thread that panics must not keep a stop from running.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Stop {
    pub fn new(mode: Mode, progress: Arc<Progress>) -> Stop {
        Stop {
            flag: AtomicBool::new(false),
            owner: Mutex::new(Owner::Nobody),
            counted: Mutex::new(Summary::default()),
            mode,
            progress,
        }
    }

    pub fn flag(&self) -> &AtomicBool {
        &self.flag
    }

    /// True when the search flow owns the output. It takes the output when
    /// nobody owns it, and a second call gives true again. False when a stop
    /// owns the output.
    pub fn claim_for_search(&self) -> bool {
        let mut owner = lock(&self.owner);
        match *owner {
            Owner::Nobody => {
                *owner = Owner::Search;
                true
            }
            Owner::Search => true,
            Owner::Stop => false,
        }
    }

    /// Write one notice of the search flow to the error stream, only when no
    /// stop owns the output. It holds the lock of `owner` while it writes, so
    /// a notice never comes after the stop failure.
    pub fn notice(&self, stderr: &mut dyn Write, text: &str) {
        let owner = lock(&self.owner);
        if *owner != Owner::Stop {
            let _ = writeln!(stderr, "{text}");
        }
    }

    /// The search flow calls it after it counts.
    pub fn count(&self, summary: &Summary) {
        lock(&self.counted).clone_from(summary);
    }

    /// Take the output for a stop. False when the search flow or an earlier
    /// stop owns it.
    fn take(&self) -> bool {
        let mut owner = lock(&self.owner);
        if *owner == Owner::Nobody {
            *owner = Owner::Stop;
            true
        } else {
            false
        }
    }

    /// Steps (a) to (c) of decision 12: the flag, the output, and the stop
    /// failure. The caller then ends jag.
    pub fn on_signal(&self, stdout: &mut dyn Write, stderr: &mut dyn Write) {
        self.flag.store(true, Ordering::SeqCst);
        if !self.take() {
            return;
        }
        let failure = Failure::from(ScoreError::Stopped);
        match self.mode {
            Mode::Text => {
                self.progress.clear_for_stop();
                let _ = write!(stderr, "{}", failure.render());
                let _ = stderr.flush();
            }
            Mode::Json => {
                let mut summary = lock(&self.counted).clone();
                let buffer = error_document(&mut summary, failure);
                let _ = stdout.write_all(&buffer).and_then(|()| stdout.flush());
            }
        }
    }

    /// Steps (a) to (c) of decision 15. True when this stop took the output,
    /// so that the caller ends jag. False when the search flow or a Ctrl-C
    /// owns the output.
    pub fn on_closed_pipe(&self) -> bool {
        self.flag.store(true, Ordering::SeqCst);
        if !self.take() {
            return false;
        }
        self.progress.stop();
        true
    }
}

/// End jag with the default action of `signal`, as grep ends. The default
/// action ends the process. If it does not, end with the shell code.
fn end_by(signal: i32) -> ! {
    let _ = emulate_default_handler(signal);
    std::process::exit(128 + signal)
}

/// Register SIGINT and start the stop thread. The thread waits for the
/// signal, calls `on_signal` with the real streams, and then ends jag by
/// SIGINT.
pub fn install(stop: &Arc<Stop>) -> io::Result<()> {
    let mut signals = Signals::new([SIGINT])?;
    let stop = Arc::clone(stop);
    std::thread::spawn(move || {
        if signals.forever().next().is_some() {
            stop.on_signal(&mut io::stdout(), &mut io::stderr());
            end_by(SIGINT);
        }
    });
    Ok(())
}

/// When standard output is a pipe or a socket, start the watch thread. When
/// the reader closes it and this stop takes the output, the thread ends jag
/// by SIGPIPE, with no message.
pub fn watch_pipe(stop: &Arc<Stop>) {
    let Ok(fd) = io::stdout().as_fd().try_clone_to_owned() else {
        return;
    };
    if !is_pipe_or_socket(fd.as_fd()) {
        return;
    }
    let stop = Arc::clone(stop);
    std::thread::spawn(move || {
        if wait_for_closed_reader(fd.as_fd(), WATCH_EVERY) && stop.on_closed_pipe() {
            end_by(SIGPIPE);
        }
    });
}

/// True when the stream is a pipe or a socket. Each error gives false.
fn is_pipe_or_socket(fd: BorrowedFd<'_>) -> bool {
    let Ok(owned) = fd.try_clone_to_owned() else {
        return false;
    };
    File::from(owned)
        .metadata()
        .map(|metadata| {
            let kind = metadata.file_type();
            kind.is_fifo() || kind.is_socket()
        })
        .unwrap_or(false)
}

/// Every `every`, ask the system about `fd`, with a zero timeout. True when
/// the reader closed the pipe or the socket. False when `poll` fails with an
/// error other than `INTR`: then jag watches no more.
fn wait_for_closed_reader(fd: BorrowedFd<'_>, every: Duration) -> bool {
    let now = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    loop {
        let mut fds = [PollFd::new(&fd, PollFlags::OUT)];
        match poll(&mut fds, Some(&now)) {
            Ok(_) => {
                if fds[0].revents().intersects(PollFlags::ERR | PollFlags::HUP) {
                    return true;
                }
            }
            Err(Errno::INTR) => {}
            Err(_) => return false,
        }
        std::thread::sleep(every);
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::sync::mpsc;
    use std::time::Instant;

    use super::*;
    use crate::output::progress::tests::Shared;

    const STOP_TEXT: &str =
        "jag: the search was stopped.\nTo get the results, run the search again.\n";

    fn stop(mode: Mode, on: bool) -> (Stop, Shared) {
        let line = Shared::default();
        let progress = Arc::new(Progress::new(Box::new(line.clone()), on));
        (Stop::new(mode, progress), line)
    }

    #[test]
    fn the_search_flow_claims_the_output_and_a_stop_after_it_writes_nothing() {
        let (stop, _) = stop(Mode::Text, false);
        assert!(stop.claim_for_search());
        assert!(stop.claim_for_search());
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        stop.on_signal(&mut stdout, &mut stderr);
        assert!(stop.flag().load(Ordering::SeqCst));
        assert!(stdout.is_empty() && stderr.is_empty());
        assert!(!stop.on_closed_pipe());
    }

    #[test]
    fn after_a_stop_the_search_flow_gets_no_output_and_no_notice() {
        let (stop, _) = stop(Mode::Text, false);
        stop.on_signal(&mut Vec::new(), &mut Vec::new());
        assert!(!stop.claim_for_search());
        let mut stderr = Vec::new();
        stop.notice(&mut stderr, "jag: a notice");
        assert!(stderr.is_empty());
    }

    #[test]
    fn a_notice_shows_while_nobody_or_the_search_flow_owns_the_output() {
        let (stop, _) = stop(Mode::Text, false);
        let mut stderr = Vec::new();
        stop.notice(&mut stderr, "one");
        assert!(stop.claim_for_search());
        stop.notice(&mut stderr, "two");
        assert_eq!(stderr, b"one\ntwo\n");
    }

    #[test]
    fn a_signal_in_text_mode_writes_the_stop_failure_to_the_error_stream() {
        let (stop, line) = stop(Mode::Text, false);
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        stop.on_signal(&mut stdout, &mut stderr);
        assert!(stdout.is_empty());
        assert_eq!(String::from_utf8(stderr).expect("text"), STOP_TEXT);
        assert_eq!(line.text(), "");
    }

    #[test]
    fn a_signal_removes_the_progress_line_before_the_failure() {
        let (stop, line) = stop(Mode::Text, true);
        stop.progress.start(70);
        stop.progress.advance(70);
        line.take();
        stop.on_signal(&mut Vec::new(), &mut Vec::new());
        assert_eq!(line.take(), format!("\r{}\r", " ".repeat(40)));
    }

    #[test]
    fn a_signal_in_structured_mode_writes_one_document_to_standard_output() {
        let (stop, line) = stop(Mode::Json, true);
        let counted = Summary {
            results_found: 200,
            results_scored: 200,
            ..Summary::default()
        };
        stop.count(&counted);
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        stop.on_signal(&mut stdout, &mut stderr);
        assert!(stderr.is_empty());
        assert_eq!(line.text(), "");
        let document: serde_json::Value = serde_json::from_slice(&stdout).expect("one document");
        assert_eq!(document["exit_code"], 130);
        assert_eq!(document["error"]["cause"], "the search was stopped.");
        assert_eq!(document["results_found"], 200);
        assert_eq!(document["results_scored"], 200);
        assert_eq!(document["results"], serde_json::json!([]));
    }

    /// A writer that tells the flag of the stop when it gets its first byte.
    struct Spy<'a> {
        flag: &'a AtomicBool,
        seen: Option<bool>,
    }

    impl Write for Spy<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.seen.get_or_insert(self.flag.load(Ordering::SeqCst));
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_signal_sets_the_flag_before_its_first_write() {
        let (stop, _) = stop(Mode::Text, false);
        let mut spy = Spy {
            flag: stop.flag(),
            seen: None,
        };
        stop.on_signal(&mut Vec::new(), &mut spy);
        assert_eq!(spy.seen, Some(true));
    }

    #[test]
    fn a_closed_pipe_sets_the_flag_removes_the_line_and_writes_nothing_else() {
        let (stop, line) = stop(Mode::Text, true);
        stop.progress.start(70);
        line.take();
        assert!(stop.on_closed_pipe());
        assert!(stop.flag().load(Ordering::SeqCst));
        assert_eq!(line.take(), format!("\r{}\r", " ".repeat(37)));
        // The line is off for good: a call that ends later writes no line.
        stop.progress.advance(32);
        assert_eq!(line.take(), "");
        assert!(!stop.claim_for_search());
    }

    #[test]
    fn a_pipe_and_a_socket_are_watched_and_a_file_and_dev_null_are_not() {
        let (_reader, writer) = io::pipe().expect("pipe");
        assert!(is_pipe_or_socket(writer.as_fd()));
        let (one, _two) = UnixStream::pair().expect("pair");
        assert!(is_pipe_or_socket(one.as_fd()));
        let file = tempfile::tempfile().expect("file");
        assert!(!is_pipe_or_socket(file.as_fd()));
        let null = File::create("/dev/null").expect("dev null");
        assert!(!is_pipe_or_socket(null.as_fd()));
    }

    #[test]
    fn the_watch_sees_the_reader_close_the_pipe() {
        let (reader, writer) = io::pipe().expect("pipe");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let closed = wait_for_closed_reader(writer.as_fd(), Duration::from_millis(20));
            let _ = sender.send(closed);
        });
        // While the reader is open, the call does not return.
        assert!(receiver.recv_timeout(Duration::from_millis(500)).is_err());
        let started = Instant::now();
        drop(reader);
        let closed = receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("the call returns");
        assert!(closed);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn the_watch_sees_the_other_end_of_a_socket_close() {
        let (one, two) = UnixStream::pair().expect("pair");
        drop(two);
        assert!(wait_for_closed_reader(
            one.as_fd(),
            Duration::from_millis(20)
        ));
    }
}
