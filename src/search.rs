//! The search of the files and folders, or of the piped text: a parallel
//! walk, a search of each file, and a count of the files that jag skips.

pub mod filter;
pub mod input;
pub mod snippet;
pub mod walk;

use std::path::{Path, PathBuf};

pub use filter::{FileFilter, GlobKind};
pub use walk::{
    FileMatches, MatchLine, ScanOptions, ScanReport, SkipCounts, WalkWarning, WarningKind, scan,
    scan_current_folder, scan_input,
};

/// Where the text of a result came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    File(PathBuf),
    StandardInput,
}

impl Source {
    /// The label of standard input, in the place of a path.
    pub const STANDARD_INPUT_LABEL: &'static str = "(standard input)";

    /// The bytes that show the source: the path as typed, or the label.
    pub fn label(&self) -> &[u8] {
        match self {
            Source::File(path) => path.as_os_str().as_encoded_bytes(),
            Source::StandardInput => Self::STANDARD_INPUT_LABEL.as_bytes(),
        }
    }

    /// The path of a file. `None` for standard input.
    pub fn path(&self) -> Option<&Path> {
        match self {
            Source::File(path) => Some(path),
            Source::StandardInput => None,
        }
    }
}
