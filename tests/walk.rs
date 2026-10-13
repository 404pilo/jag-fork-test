//! Tests of the folder search, over real folders in a temporary folder.
//!
//! Each test calls `scan` with the two global ignore options off, so the
//! global git ignore file and the ignore files above the temporary folder do
//! not change the counts. No test changes a variable of the process.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use grep_regex::RegexMatcher;
use jag::search::{ScanOptions, ScanReport, SkipCounts, WalkWarning, WarningKind, scan};
use tempfile::TempDir;

fn write(root: &Path, relative: &str, text: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("make folders");
    fs::write(path, text).expect("write file");
}

fn folder(root: &Path, relative: &str) {
    fs::create_dir_all(root.join(relative)).expect("make folder");
}

fn options() -> ScanOptions {
    ScanOptions {
        global_ignore: false,
        parent_ignore: false,
        ..ScanOptions::default()
    }
}

fn run(root: &Path) -> ScanReport {
    run_roots(&[root.to_path_buf()])
}

fn run_roots(roots: &[PathBuf]) -> ScanReport {
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    scan(roots, &matcher, &options())
}

/// The paths of the files with a match, relative to the root.
fn names(report: &ScanReport, root: &Path) -> Vec<String> {
    report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(root)
                .expect("a path under the root")
                .display()
                .to_string()
        })
        .collect()
}

/// The tree of CRITERION-23 and CRITERION-24.
fn noise_tree() -> TempDir {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, ".hidden.txt", b"needle\n");
    write(root, ".cache/e.txt", b"needle\n");
    folder(root, ".git");
    write(root, ".gitignore", b"build/\n*.log\n");
    write(root, "build/f.txt", b"needle\n");
    write(root, "g.log", b"needle\n");
    write(root, "bin.dat", b"needle\n\0");
    write(root, "ok.txt", b"needle\n");
    dir
}

#[test]
fn default_options_read_both_ignore_sources() {
    let defaults = ScanOptions::default();
    assert_eq!(defaults.threads, 0);
    assert!(defaults.global_ignore);
    assert!(defaults.parent_ignore);
}

#[test]
fn criterion_22_searches_each_subfolder() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    for file in ["a.txt", "x/b.txt", "x/y/c.txt", "x/y/z/d.txt"] {
        write(root, file, b"needle\n");
    }
    let report = run(root);
    assert_eq!(
        names(&report, root),
        ["a.txt", "x/b.txt", "x/y/c.txt", "x/y/z/d.txt"]
    );
    for file in &report.files {
        assert_eq!(file.lines.len(), 1);
        assert_eq!(file.lines[0].line_number, 1);
        assert_eq!(file.lines[0].text, b"needle");
    }
    assert_eq!(report.skipped.total(), 0);
    assert!(report.warnings.is_empty());
}

#[test]
fn a_match_has_its_line_number_and_no_line_terminator() {
    let dir = TempDir::new().expect("temporary folder");
    write(
        dir.path(),
        "a.txt",
        b"one\r\ntwo needle\r\nthree\nneedle four\n",
    );
    let report = run(dir.path());
    let lines = &report.files[0].lines;
    assert_eq!(lines.len(), 2);
    assert_eq!(
        (lines[0].line_number, lines[0].text.as_slice()),
        (2, b"two needle".as_slice())
    );
    assert_eq!(
        (lines[1].line_number, lines[1].text.as_slice()),
        (4, b"needle four".as_slice())
    );
}

#[test]
fn a_file_with_no_match_is_not_in_the_result() {
    let dir = TempDir::new().expect("temporary folder");
    write(dir.path(), "a.txt", b"nothing here\n");
    let report = run(dir.path());
    assert!(report.files.is_empty());
    assert_eq!(report.skipped.total(), 0);
}

#[test]
fn criterion_23_skips_noise_files() {
    let dir = noise_tree();
    let report = run(dir.path());
    assert_eq!(names(&report, dir.path()), ["ok.txt"]);
}

#[test]
fn criterion_24_counts_the_skipped_files() {
    let dir = noise_tree();
    let report = run(dir.path());
    assert_eq!(
        report.skipped,
        SkipCounts {
            binary: 1,
            hidden: 4,
            ignored: 2
        }
    );
    assert_eq!(
        report.skipped.notice().as_deref(),
        Some("jag: skipped 7 files and folders (1 binary, 4 hidden, 2 ignored)")
    );
    assert!(report.warnings.is_empty());
}

#[test]
fn the_notice_is_none_when_nothing_is_skipped() {
    assert_eq!(SkipCounts::default().notice(), None);
    assert_eq!(SkipCounts::default().total(), 0);
}

#[test]
fn a_skipped_folder_counts_as_one() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    folder(root, ".git");
    write(root, ".gitignore", b"build/\n");
    for number in 0..50 {
        write(root, &format!("build/f{number}.txt"), b"needle\n");
    }
    write(root, "ok.txt", b"needle\n");
    let report = run(root);
    assert_eq!(names(&report, root), ["ok.txt"]);
    assert_eq!(
        report.skipped,
        SkipCounts {
            binary: 0,
            hidden: 2,
            ignored: 1
        }
    );
}

#[test]
fn an_ignore_file_works_with_no_git_folder() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, ".ignore", b"*.tmp\n");
    write(root, "t.tmp", b"needle\n");
    write(root, "ok.txt", b"needle\n");
    let report = run(root);
    assert_eq!(names(&report, root), ["ok.txt"]);
    assert_eq!(
        report.skipped,
        SkipCounts {
            binary: 0,
            hidden: 1,
            ignored: 1
        }
    );
}

#[test]
fn a_gitignore_does_not_work_with_no_git_folder() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, ".gitignore", b"*.log\n");
    write(root, "g.log", b"needle\n");
    let report = run(root);
    assert_eq!(names(&report, root), ["g.log"]);
}

#[test]
fn a_nul_byte_after_a_match_makes_the_file_binary() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    let mut text = b"needle\n".to_vec();
    text.extend(std::iter::repeat_n(b'x', 100 * 1024));
    text.extend(b"\n\0");
    write(root, "late.dat", &text);
    let report = run(root);
    assert!(report.files.is_empty());
    assert_eq!(
        report.skipped,
        SkipCounts {
            binary: 1,
            hidden: 0,
            ignored: 0
        }
    );
}

#[test]
fn a_hidden_folder_that_a_rule_whitelists_is_searched() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, ".ignore", b"*.tmp\n!.github/\n");
    write(root, ".github/w.yml", b"needle\n");
    write(root, ".env", b"needle\n");
    write(root, "t.tmp", b"needle\n");
    write(root, "ok.txt", b"needle\n");
    let report = run(root);
    assert_eq!(names(&report, root), [".github/w.yml", "ok.txt"]);
    assert_eq!(
        report.skipped,
        SkipCounts {
            binary: 0,
            hidden: 2,
            ignored: 1
        }
    );
}

#[test]
fn a_bad_rule_gives_one_warning_and_the_walk_goes_on() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    // The crate accepts `a[`; a reversed range is a rule that it rejects.
    write(root, ".ignore", b"[z-a]\n");
    write(root, "ok.txt", b"needle\n");
    let report = run(root);
    assert_eq!(names(&report, root), ["ok.txt"]);
    assert_eq!(
        report.warnings,
        [WalkWarning {
            path: Some(root.join(".ignore")),
            kind: WarningKind::IgnoreRule
        }]
    );
}

#[test]
fn the_warning_messages_have_the_text_of_the_table() {
    let path = Some(PathBuf::from("some/path"));
    let read = WarningKind::Read(io::ErrorKind::PermissionDenied);
    let message = |path: &Option<PathBuf>, kind| {
        WalkWarning {
            path: path.clone(),
            kind,
        }
        .message()
    };
    assert_eq!(
        message(&path, read),
        "jag: cannot read some/path: permission denied"
    );
    assert_eq!(
        message(&None, read),
        "jag: cannot read a file: permission denied"
    );
    assert_eq!(
        message(&path, WarningKind::IgnoreRule),
        "jag: a rule in some/path is not valid; jag uses the other rules"
    );
    assert_eq!(
        message(&None, WarningKind::IgnoreRule),
        "jag: an ignore rule is not valid; jag uses the other rules"
    );
    assert_eq!(
        message(&path, WarningKind::Other),
        "jag: cannot walk some/path"
    );
    assert_eq!(
        message(&None, WarningKind::Other),
        "jag: cannot walk a folder"
    );
}

#[test]
fn a_root_that_is_a_file_is_searched() {
    let dir = TempDir::new().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    let file = dir.path().join("a.txt");
    let report = run_roots(std::slice::from_ref(&file));
    assert_eq!(report.files.len(), 1);
    assert_eq!(report.files[0].path, file);
}

#[test]
fn no_root_gives_an_empty_report() {
    let report = run_roots(&[]);
    assert!(report.files.is_empty());
    assert_eq!(report.skipped, SkipCounts::default());
    assert!(report.warnings.is_empty());
}

#[test]
fn the_result_is_the_same_on_each_run() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    for number in 0..200 {
        write(root, &format!("d{}/f{number}.txt", number % 7), b"needle\n");
    }
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    let options = ScanOptions {
        threads: 8,
        ..options()
    };
    let first = scan(&[root.to_path_buf()], &matcher, &options);
    assert_eq!(first.files.len(), 200);
    for _ in 0..20 {
        assert_eq!(scan(&[root.to_path_buf()], &matcher, &options), first);
    }
}

#[cfg(unix)]
mod unix {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use super::*;

    #[test]
    fn an_unreadable_folder_gives_a_warning_and_is_not_skipped() {
        let dir = TempDir::new().expect("temporary folder");
        let root = dir.path();
        folder(root, ".git");
        write(root, ".gitignore", b"*.log\n");
        write(root, "g.log", b"needle\n");
        write(root, "ok.txt", b"needle\n");
        folder(root, "locked");
        let locked = root.join("locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("lock folder");
        // A user with the right to read any folder (root) cannot run this test.
        let can_read = fs::read_dir(&locked).is_ok();
        let report = run(root);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock folder");
        if can_read {
            return;
        }
        assert_eq!(names(&report, root), ["ok.txt"]);
        assert_eq!(
            report.warnings,
            [WalkWarning {
                path: Some(locked),
                kind: WarningKind::Read(io::ErrorKind::PermissionDenied)
            }]
        );
        assert_eq!(
            report.skipped,
            SkipCounts {
                binary: 0,
                hidden: 2,
                ignored: 1
            }
        );
    }

    #[test]
    fn a_link_below_the_root_is_not_searched_and_not_counted() {
        let dir = TempDir::new().expect("temporary folder");
        let root = dir.path();
        write(root, "target.txt", b"needle\n");
        symlink(root.join("target.txt"), root.join("link.txt")).expect("make link");
        let report = run(root);
        assert_eq!(names(&report, root), ["target.txt"]);
        assert_eq!(report.skipped.total(), 0);
    }

    #[test]
    fn a_root_that_is_a_link_to_a_file_is_searched() {
        let dir = TempDir::new().expect("temporary folder");
        let root = dir.path();
        write(root, "target.txt", b"needle\n");
        let link = root.join("link.txt");
        symlink(root.join("target.txt"), &link).expect("make link");
        let report = run_roots(std::slice::from_ref(&link));
        assert_eq!(report.files.len(), 1);
        assert_eq!(report.files[0].path, link);
    }

    #[test]
    fn a_root_that_is_a_link_to_a_folder_is_walked() {
        let dir = TempDir::new().expect("temporary folder");
        let root = dir.path();
        write(root, "real/a.txt", b"needle\n");
        write(root, "real/x/b.txt", b"needle\n");
        let link = root.join("link");
        symlink(root.join("real"), &link).expect("make link");
        let report = run_roots(std::slice::from_ref(&link));
        assert_eq!(names(&report, &link), ["a.txt", "x/b.txt"]);
        for file in &report.files {
            assert!(file.path.starts_with(&link));
        }
    }
}
