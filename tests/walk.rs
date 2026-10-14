//! Tests of the folder search, over real folders in a temporary folder.
//!
//! Each test calls `scan` with the two global ignore options off, so the
//! global git ignore file and the ignore files above the temporary folder do
//! not change the counts. No test changes a variable of the process.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use grep_regex::RegexMatcher;
use jag::search::{
    FileFilter, GlobKind, ScanOptions, ScanReport, SkipCounts, Source, WalkWarning, WarningKind,
    scan,
};
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
            file.source
                .path()
                .expect("a file")
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
    assert_eq!(report.files[0].source, Source::File(file));
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
        assert_eq!(report.files[0].source, Source::File(link));
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
            assert!(
                file.source
                    .path()
                    .is_some_and(|path| path.starts_with(&link))
            );
        }
    }
}

#[test]
fn files_come_in_the_order_of_the_paths_on_the_command_line() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "a/2.txt", b"needle\n");
    write(root, "a/10.txt", b"needle\n");
    write(root, "b/1.txt", b"needle\n");
    let report = run_roots(&[root.join("b"), root.join("a")]);
    assert_eq!(names(&report, root), ["b/1.txt", "a/10.txt", "a/2.txt"]);
    let report = run_roots(&[root.join("a"), root.join("b")]);
    assert_eq!(names(&report, root), ["a/10.txt", "a/2.txt", "b/1.txt"]);
}

fn run_eight_threads(roots: &[PathBuf]) -> ScanReport {
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    let options = ScanOptions {
        threads: 8,
        ..options()
    };
    scan(roots, &matcher, &options)
}

// CRITERION-61 and decision 6 of FEATURE-14: the order of the paths on the
// command line, then the byte order inside each path, with 8 threads.
#[test]
fn files_and_folders_come_in_the_order_of_the_command_line_with_8_threads() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    for file in ["z.txt", "dir/b.txt", "dir/a.txt", "a.txt"] {
        write(root, file, b"needle\n");
    }
    let roots = [root.join("z.txt"), root.join("dir"), root.join("a.txt")];
    for _ in 0..20 {
        let report = run_eight_threads(&roots);
        assert_eq!(
            names(&report, root),
            ["z.txt", "dir/a.txt", "dir/b.txt", "a.txt"]
        );
    }
}

// Decision 6 of FEATURE-14: a path two times and paths that overlap, as
// `grep -r needle dir dir/sub dir`.
#[test]
fn a_path_two_times_and_paths_that_overlap_keep_the_order_with_8_threads() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "dir/a.txt", b"needle\n");
    write(root, "dir/sub/c.txt", b"needle\n");
    let roots = [root.join("dir"), root.join("dir/sub"), root.join("dir")];
    for _ in 0..20 {
        let report = run_eight_threads(&roots);
        assert_eq!(
            names(&report, root),
            [
                "dir/a.txt",
                "dir/sub/c.txt",
                "dir/sub/c.txt",
                "dir/a.txt",
                "dir/sub/c.txt"
            ]
        );
    }
}

// Decision 7 of FEATURE-14: each path that does not exist gives one warning,
// in the order of the command line.
#[test]
fn paths_that_do_not_exist_give_warnings_in_the_order_of_the_command_line() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "a.txt", b"needle\n");
    let roots = [
        root.join("nofile2"),
        root.join("a.txt"),
        root.join("nofile1"),
    ];
    for _ in 0..20 {
        let report = run_eight_threads(&roots);
        assert_eq!(names(&report, root), ["a.txt"]);
        assert_eq!(
            report.warnings,
            [
                WalkWarning {
                    path: Some(root.join("nofile2")),
                    kind: WarningKind::Missing,
                },
                WalkWarning {
                    path: Some(root.join("nofile1")),
                    kind: WarningKind::Missing,
                },
            ]
        );
        assert!(report.path_missing());
        assert_eq!(report.skipped, SkipCounts::default());
    }
}

// FEATURE-13: the globs of --include and --exclude, and the files with no match.
fn filtered(globs: &[(GlobKind, &str)]) -> ScanOptions {
    let globs: Vec<(GlobKind, String)> = globs
        .iter()
        .map(|(kind, glob)| (*kind, (*glob).to_owned()))
        .collect();
    ScanOptions {
        files: FileFilter::new(&globs).expect("valid globs"),
        ..options()
    }
}

#[test]
fn an_include_glob_searches_only_the_files_that_match_and_the_skip_counts_do_not_change() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "a.txt", b"needle\n");
    write(root, "dir/c.rs", b"needle\n");
    write(root, ".hidden.rs", b"needle\n");
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    let plain = scan(&[root.to_path_buf()], &matcher, &options());
    let report = scan(
        &[root.to_path_buf()],
        &matcher,
        &filtered(&[(GlobKind::Include, "*.rs")]),
    );
    assert_eq!(names(&report, root), ["dir/c.rs"]);
    assert_eq!(report.skipped, plain.skipped);
    assert_eq!(report.skipped.hidden, 1);
}

#[test]
fn a_glob_applies_to_a_file_that_the_user_names() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "a.txt", b"needle\n");
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    let file = root.join("a.txt");
    let skipped = scan(
        std::slice::from_ref(&file),
        &matcher,
        &filtered(&[(GlobKind::Include, "*.rs")]),
    );
    assert!(skipped.files.is_empty());
    let kept = scan(
        std::slice::from_ref(&file),
        &matcher,
        &filtered(&[(GlobKind::Exclude, "*.rs")]),
    );
    assert_eq!(kept.files.len(), 1);
}

#[test]
fn a_file_with_no_match_is_kept_only_when_the_option_says_so_and_a_binary_file_never() {
    let dir = TempDir::new().expect("temporary folder");
    let root = dir.path();
    write(root, "a.txt", b"needle\nneedle\n");
    write(root, "b.txt", b"nothing\n");
    write(root, "bin.dat", b"needle\n\0");
    let matcher = RegexMatcher::new("needle").expect("a valid pattern");
    let report = scan(&[root.to_path_buf()], &matcher, &options());
    assert_eq!(names(&report, root), ["a.txt"]);
    let keep = ScanOptions {
        keep_files_with_no_match: true,
        ..options()
    };
    let report = scan(&[root.to_path_buf()], &matcher, &keep);
    assert_eq!(names(&report, root), ["a.txt", "b.txt"]);
    assert_eq!(report.files[0].lines.len(), 2);
    assert!(report.files[1].lines.is_empty());
    assert_eq!(report.skipped.binary, 1);
}
