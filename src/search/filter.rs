//! The globs of `--include` and `--exclude` (FEATURE-13, decisions 25 to 27).
//!
//! A glob matches the name of a file: the last part of its path. The last
//! glob of the command line that matches decides, as in GNU grep.

use std::ffi::OsStr;

use globset::{Glob, GlobMatcher};

/// The option that gave a glob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlobKind {
    Include,
    Exclude,
}

/// The globs of --include and --exclude, in the order of the command line
/// (FEATURE-13, decision 25).
#[derive(Clone, Debug, Default)]
pub struct FileFilter {
    globs: Vec<(GlobKind, GlobMatcher)>,
}

impl FileFilter {
    /// The filter of `globs`. A glob that does not parse gives its kind, and
    /// no text of the glob (decision 26).
    pub fn new(globs: &[(GlobKind, String)]) -> Result<FileFilter, GlobKind> {
        let globs = globs
            .iter()
            .map(|(kind, glob)| {
                Glob::new(glob)
                    .map(|glob| (*kind, glob.compile_matcher()))
                    .map_err(|_| *kind)
            })
            .collect::<Result<_, _>>()?;
        Ok(FileFilter { globs })
    }

    /// True when jag searches a file with this name: the last glob that
    /// matches decides. When no glob matches, jag searches the file, unless
    /// the first glob is an --include glob.
    pub fn allows(&self, name: &OsStr) -> bool {
        match self
            .globs
            .iter()
            .rev()
            .find(|(_, glob)| glob.is_match(name))
        {
            Some((kind, _)) => *kind == GlobKind::Include,
            None => self
                .globs
                .first()
                .is_none_or(|(kind, _)| *kind == GlobKind::Exclude),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GlobKind::{Exclude, Include};

    fn filter(globs: &[(GlobKind, &str)]) -> FileFilter {
        let globs: Vec<(GlobKind, String)> = globs
            .iter()
            .map(|(kind, glob)| (*kind, (*glob).to_owned()))
            .collect();
        FileFilter::new(&globs).expect("valid globs")
    }

    fn allows(filter: &FileFilter, name: &str) -> bool {
        filter.allows(OsStr::new(name))
    }

    #[test]
    fn no_glob_allows_each_name() {
        let filter = FileFilter::default();
        assert!(allows(&filter, "a.txt"));
        assert!(allows(&filter, ".hidden"));
    }

    #[test]
    fn the_last_glob_that_matches_decides() {
        let filter = filter(&[(Exclude, "*.md"), (Include, "*.rs")]);
        assert!(allows(&filter, "a.txt"));
        assert!(allows(&filter, "c.rs"));
        assert!(!allows(&filter, "d.md"));
        let filter = self::filter(&[(Include, "*.rs"), (Exclude, "c.*")]);
        assert!(!allows(&filter, "c.rs"));
        assert!(allows(&filter, "b.rs"));
    }

    #[test]
    fn a_first_include_glob_refuses_each_name_that_no_glob_matches() {
        let filter = filter(&[(Include, "*.rs")]);
        assert!(!allows(&filter, "a.txt"));
        assert!(allows(&filter, "c.rs"));
    }

    #[test]
    fn a_glob_uses_the_rules_of_ripgrep_and_is_case_sensitive() {
        let filter = filter(&[(Include, "*.{rs,md}")]);
        assert!(allows(&filter, "d.md"));
        assert!(allows(&filter, "c.rs"));
        assert!(!allows(&filter, "c.RS"));
    }

    #[test]
    fn a_glob_that_does_not_parse_gives_its_kind() {
        let globs = [(Include, "[".to_owned())];
        assert_eq!(FileFilter::new(&globs).err(), Some(Include));
        let globs = [(Include, "*.rs".to_owned()), (Exclude, "a{".to_owned())];
        assert_eq!(FileFilter::new(&globs).err(), Some(Exclude));
    }
}
