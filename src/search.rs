//! The search of a folder: a parallel walk, a search of each file, and a count
//! of the files that jag skips.

pub mod filter;
pub mod snippet;
pub mod walk;

pub use filter::{FileFilter, GlobKind};
pub use walk::{
    FileMatches, MatchLine, ScanOptions, ScanReport, SkipCounts, WalkWarning, WarningKind, scan,
};
