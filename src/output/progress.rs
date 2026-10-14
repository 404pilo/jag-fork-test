//! The progress line of the error stream (STORY-41, DECISION-37).

use std::io::Write;
use std::sync::{Mutex, MutexGuard};

/// The progress line of the error stream. It writes only when it is on: in
/// text mode, when the error stream is a terminal. The 6 threads of `score`
/// and a stop can share it.
pub struct Progress {
    state: Mutex<State>,
}

struct State {
    out: Box<dyn Write + Send>,
    /// False when the line is off, and after a stop. Off is final.
    on: bool,
    total: usize,
    done: usize,
    /// The count of characters of the line on the screen. 0 for no line.
    width: usize,
    /// The count of characters of the longest line since `start`. `clear`
    /// does not change it.
    widest: usize,
}

impl State {
    /// The line, with no `\r` and no `\n`.
    fn line(&self) -> String {
        format!(
            "jag: the model scored {} of {} results",
            self.done, self.total
        )
    }

    /// Write `text`. A write error is ignored, as the write error of a notice.
    fn put(&mut self, text: &str) {
        let _ = self
            .out
            .write_all(text.as_bytes())
            .and_then(|()| self.out.flush());
    }

    /// Write the line again, after `prefix`.
    fn show(&mut self, prefix: &str) {
        let line = self.line();
        let width = line.chars().count();
        self.put(&format!("{prefix}{line}"));
        self.width = width;
        self.widest = self.widest.max(width);
    }

    /// Remove the line: `\r`, `width` spaces, `\r`. No text when no line shows.
    fn clear(&mut self) {
        if !self.on || self.width == 0 {
            return;
        }
        self.blank(self.width);
        self.width = 0;
    }

    fn blank(&mut self, width: usize) {
        self.put(&format!("\r{}\r", " ".repeat(width)));
    }
}

impl Progress {
    pub fn new(out: Box<dyn Write + Send>, on: bool) -> Progress {
        Progress {
            state: Mutex::new(State {
                out,
                on,
                total: 0,
                done: 0,
                width: 0,
                widest: 0,
            }),
        }
    }

    /// A thread of `score` that panics must not stop the others from writing.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Write `jag: the model scored 0 of N results`.
    pub fn start(&self, total: usize) {
        let mut state = self.state();
        if !state.on {
            return;
        }
        state.total = total;
        state.done = 0;
        state.show("");
    }

    /// Add the results of one call, and write the line again after `\r`.
    pub fn advance(&self, count: usize) {
        let mut state = self.state();
        if !state.on {
            return;
        }
        state.done += count;
        state.show("\r");
    }

    /// Remove the line: `\r`, `width` spaces, `\r`. No text when no line
    /// shows.
    pub fn clear(&self) {
        self.state().clear();
    }

    /// For a Ctrl-C: `\r`, `widest + 2` spaces, `\r`, also when no line
    /// shows, so that the `^C` of the terminal goes too. Then the line is
    /// off.
    pub fn clear_for_stop(&self) {
        let mut state = self.state();
        if !state.on {
            return;
        }
        let width = state.widest + 2;
        state.blank(width);
        state.width = 0;
        state.on = false;
    }

    /// For a closed pipe: `clear`, then the line is off for good. A call
    /// that ends after the stop writes no line.
    pub fn stop(&self) {
        let mut state = self.state();
        state.clear();
        state.on = false;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io;
    use std::sync::Arc;

    use super::*;

    /// A writer that the test can read while the `Progress` holds it.
    #[derive(Clone, Default)]
    pub(crate) struct Shared(pub(crate) Arc<Mutex<Vec<u8>>>);

    impl Shared {
        pub(crate) fn text(&self) -> String {
            String::from_utf8(self.0.lock().expect("lock").clone()).expect("text")
        }

        /// The text since the last call, with the buffer empty again.
        pub(crate) fn take(&self) -> String {
            let bytes = std::mem::take(&mut *self.0.lock().expect("lock"));
            String::from_utf8(bytes).expect("text")
        }
    }

    impl Write for Shared {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().expect("lock").extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn progress(on: bool) -> (Progress, Shared) {
        let out = Shared::default();
        (Progress::new(Box::new(out.clone()), on), out)
    }

    #[test]
    fn the_line_counts_the_results_of_the_calls_and_clear_removes_it() {
        let (progress, out) = progress(true);
        progress.start(70);
        assert_eq!(out.take(), "jag: the model scored 0 of 70 results");
        progress.advance(32);
        assert_eq!(out.take(), "\rjag: the model scored 32 of 70 results");
        progress.advance(32);
        progress.advance(6);
        assert!(
            out.text()
                .ends_with("\rjag: the model scored 70 of 70 results")
        );
        out.take();
        progress.clear();
        assert_eq!(out.take(), format!("\r{}\r", " ".repeat(38)));
        progress.clear();
        assert_eq!(out.take(), "");
    }

    #[test]
    fn an_off_line_writes_nothing() {
        let (progress, out) = progress(false);
        progress.start(70);
        progress.advance(32);
        progress.clear();
        progress.clear_for_stop();
        progress.stop();
        assert_eq!(out.text(), "");
    }

    #[test]
    fn six_threads_give_the_last_text_the_full_count() {
        let (progress, out) = progress(true);
        progress.start(600);
        std::thread::scope(|scope| {
            for _ in 0..6 {
                scope.spawn(|| {
                    for _ in 0..100 {
                        progress.advance(1);
                    }
                });
            }
        });
        let text = out.text();
        let last = text.rsplit('\r').next().expect("a line");
        assert_eq!(last, "jag: the model scored 600 of 600 results");
    }

    #[test]
    fn clear_for_stop_removes_the_longest_line_and_the_cut_of_the_terminal() {
        let (progress, out) = progress(true);
        progress.start(70);
        progress.advance(70);
        progress.clear();
        out.take();
        progress.clear_for_stop();
        assert_eq!(out.take(), format!("\r{}\r", " ".repeat(40)));

        // With no line since `start`, the width is 2 (the `^C`).
        let (progress, out) = self::progress(true);
        progress.clear_for_stop();
        assert_eq!(out.take(), "\r  \r");
    }

    #[test]
    fn no_line_comes_after_a_stop() {
        for stop_with in [Progress::clear_for_stop, Progress::stop] {
            let (progress, out) = progress(true);
            progress.start(70);
            stop_with(&progress);
            out.take();
            progress.advance(32);
            progress.start(70);
            progress.clear();
            assert_eq!(out.take(), "", "text after the stop");
            assert!(!out.text().contains("jag: the model scored"));
        }
    }

    #[test]
    fn stop_removes_a_line_that_shows_and_writes_no_more() {
        let (progress, out) = progress(true);
        progress.start(70);
        out.take();
        progress.stop();
        assert_eq!(out.take(), format!("\r{}\r", " ".repeat(37)));
        progress.stop();
        assert_eq!(out.take(), "");
    }
}
