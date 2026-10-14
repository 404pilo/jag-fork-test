//! The input of a search with no path: the text of a pipe, or the current
//! folder (FEATURE-14, decisions 2, 3, and 12).

use std::fs::File;
use std::io::{self, IsTerminal, Read};
use std::os::fd::AsFd;
use std::os::unix::fs::FileTypeExt;

use super::walk::{WalkWarning, WarningKind};

/// What a search with no path searches (decisions 2, 3, and 12).
#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    /// The text of standard input, also when it has no byte.
    PipedText(Vec<u8>),
    /// Standard input gave an error. jag searches nothing.
    ReadError(WalkWarning),
    /// Standard input cannot give text: jag searches the current folder.
    CurrentFolder,
}

/// True when standard input can give text (decision 2): it is not a
/// terminal, and it is a regular file, a pipe, or a socket. This is the test
/// of ripgrep. A device that is not a terminal, for example `/dev/zero`,
/// never ends, so the type test stays.
pub fn stdin_can_give_text() -> bool {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return false;
    }
    stdin
        .as_fd()
        .try_clone_to_owned()
        .is_ok_and(|fd| is_text_source(&File::from(fd)))
}

/// True when `file` is a regular file, a pipe, or a socket. Each error gives
/// false.
pub fn is_text_source(file: &File) -> bool {
    file.metadata().is_ok_and(|metadata| {
        let kind = metadata.file_type();
        kind.is_file() || kind.is_fifo() || kind.is_socket()
    })
}

/// Read all of `reader`. Zero bytes give `PipedText` with no byte
/// (decision 3). An error gives `ReadError`, and jag keeps no byte.
pub fn read_input(mut reader: impl Read) -> Input {
    let mut bytes = Vec::new();
    match reader.read_to_end(&mut bytes) {
        Ok(_) => Input::PipedText(bytes),
        Err(error) => Input::ReadError(WalkWarning {
            path: None,
            kind: WarningKind::StandardInput(error.kind()),
        }),
    }
}

/// The text of standard input when it can give text, else the current
/// folder.
pub fn no_path_input() -> Input {
    if stdin_can_give_text() {
        read_input(io::stdin().lock())
    } else {
        Input::CurrentFolder
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    use super::*;

    #[test]
    fn a_device_and_a_folder_cannot_give_text() {
        let null = File::open("/dev/null").expect("open /dev/null");
        assert!(!is_text_source(&null));
        let dir = tempfile::tempdir().expect("temporary folder");
        let folder = File::open(dir.path()).expect("open folder");
        assert!(!is_text_source(&folder));
    }

    #[test]
    fn a_file_a_pipe_and_a_socket_can_give_text() {
        let file = tempfile::tempfile().expect("temporary file");
        assert!(is_text_source(&file));
        let (reader, _writer) = io::pipe().expect("pipe");
        assert!(is_text_source(&File::from(std::os::fd::OwnedFd::from(
            reader
        ))));
        let (one, _other) = UnixStream::pair().expect("socket pair");
        assert!(is_text_source(&File::from(std::os::fd::OwnedFd::from(one))));
    }

    #[test]
    fn zero_bytes_give_piped_text_with_no_byte() {
        assert_eq!(read_input(&b""[..]), Input::PipedText(Vec::new()));
    }

    #[test]
    fn bytes_give_piped_text() {
        assert_eq!(read_input(&b"x"[..]), Input::PipedText(b"x".to_vec()));
        let (reader, mut writer) = io::pipe().expect("pipe");
        writer.write_all(b"a\nb\n").expect("write");
        drop(writer);
        assert_eq!(read_input(reader), Input::PipedText(b"a\nb\n".to_vec()));
    }

    struct Failing;

    impl Read for Failing {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }

    #[test]
    fn a_reader_that_fails_gives_a_read_error_with_no_path() {
        assert_eq!(
            read_input(Failing),
            Input::ReadError(WalkWarning {
                path: None,
                kind: WarningKind::StandardInput(io::ErrorKind::BrokenPipe),
            })
        );
    }
}
