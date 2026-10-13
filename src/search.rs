//! The search of a folder: a parallel walk, a search of each file, and a count
//! of the files that jag skips.

pub mod walk;

pub use walk::{
    FileMatches, MatchLine, ScanOptions, ScanReport, SkipCounts, WalkWarning, WarningKind, scan,
};
