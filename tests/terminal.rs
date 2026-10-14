//! End-to-end tests of the progress line and of color (FEATURE-15,
//! CRITERION-62 and CRITERION-63), against the fake server.
//!
//! jag runs in pipes only here: the terminal path has unit tests (decision
//! 19). Each test sets HOME to a temporary folder and uses the made-up key.

mod common;

use std::path::Path;

use common::fake_server::FakeServer;
use common::{KEY, jag, write};
use serde_json::Value;
use tempfile::TempDir;

const QUESTION: &str = "which line says how to retry";
const USAGE_ERROR: &str = "jag: the command line is not valid. Run jag --help to see the usage.\n";

/// The bytes of `grep --color=always -n -H needle a.txt` for the line
/// `x needle y`.
const COLORED: &str = "\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[32m\x1b[K1\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[Kx \x1b[01;31m\x1b[Kneedle\x1b[m\x1b[K y\n";
const PLAIN: &str = "a.txt:1:x needle y\n";

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run `jag OPTIONS needle QUESTION a.txt` in a folder that holds `a.txt`
/// (the line `x needle y`).
fn run(server: &FakeServer, args: &[&str]) -> Run {
    let root = tempfile::tempdir().expect("folder");
    write(root.path(), "a.txt", b"x needle y\n");
    run_in(server, root.path(), args, &["needle", QUESTION, "a.txt"])
}

fn run_in(server: &FakeServer, cwd: &Path, options: &[&str], rest: &[&str]) -> Run {
    let home = tempfile::tempdir().expect("home");
    let output = jag(home.path(), &server.base_url())
        .current_dir(cwd)
        .args(options)
        .args(rest)
        .output()
        .expect("run jag");
    let run = Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    assert!(!run.stdout.contains(KEY) && !run.stderr.contains(KEY));
    run
}

/// 70 files with one match each: the model needs 3 calls.
fn seventy_results() -> TempDir {
    let root = tempfile::tempdir().expect("folder");
    for index in 0..70 {
        write(root.path(), &format!("dir/f{index}.txt"), b"needle\n");
    }
    root
}

#[test]
fn criterion_62_a_pipe_gets_no_progress_line() {
    let server = FakeServer::constant(0.9);
    let root = seventy_results();
    let run = run_in(&server, root.path(), &[], &["needle", QUESTION, "dir"]);
    assert_eq!(run.code, Some(0));
    assert!(!run.stderr.contains('\r'), "{:?}", run.stderr);
    assert!(!run.stderr.contains("the model scored"), "{:?}", run.stderr);
    assert_eq!(server.requests().len(), 3);
}

#[test]
fn criterion_62_structured_mode_gets_no_progress_line() {
    let server = FakeServer::constant(0.9);
    let root = seventy_results();
    let run = run_in(
        &server,
        root.path(),
        &["--json"],
        &["needle", QUESTION, "dir"],
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, "");
}

#[test]
fn criterion_63_always_colors_in_a_pipe_as_grep_does() {
    let server = FakeServer::constant(0.9);
    let run = run(&server, &["--color=always"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, COLORED);
    assert_eq!(run.stderr, "");
}

#[test]
fn criterion_63_colour_is_an_alias() {
    let server = FakeServer::constant(0.9);
    let run = run(&server, &["--colour=always"]);
    assert_eq!(run.stdout, COLORED);
}

#[test]
fn criterion_63_a_pipe_gets_no_color_unless_always_is_the_last_choice() {
    let server = FakeServer::constant(0.9);
    for options in [
        &[][..],
        &["--color"],
        &["--color=auto"],
        &["--color=never"],
        &["--color=always", "--color=never"],
    ] {
        let run = run(&server, options);
        assert_eq!(run.code, Some(0), "{options:?}");
        assert_eq!(run.stdout, PLAIN, "{options:?}");
    }
    let run = run(&server, &["--color=never", "--color=always"]);
    assert_eq!(run.stdout, COLORED);
}

#[test]
fn criterion_63_a_bad_value_is_an_argument_error_with_no_call() {
    let server = FakeServer::constant(0.9);
    let run = run(&server, &["--color=sometimes"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, USAGE_ERROR);
    assert!(server.requests().is_empty());

    let run = self::run(&server, &["--json", "--color=sometimes"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stderr, "");
    let document: Value = serde_json::from_str(&run.stdout).expect("a document");
    assert_eq!(document["exit_code"], 2);
    assert!(!run.stdout.contains("sometimes"));
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_63_color_conflicts_with_setup_check_and_forget_key() {
    let server = FakeServer::constant(0.9);
    for option in ["--setup", "--check", "--forget-key"] {
        let root = tempfile::tempdir().expect("folder");
        let run = run_in(&server, root.path(), &["--color=always", option], &[]);
        assert_eq!(run.code, Some(2), "{option}");
        assert_eq!(run.stderr, USAGE_ERROR, "{option}");
    }
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_63_structured_mode_has_no_color() {
    let server = FakeServer::constant(0.9);
    let run = run(&server, &["--json", "--color=always"]);
    assert_eq!(run.code, Some(0));
    assert!(!run.stdout.contains('\x1b'), "{:?}", run.stdout);
    let document: Value = serde_json::from_str(&run.stdout).expect("a document");
    assert_eq!(document["results"][0]["lines"][0]["text"], "x needle y");
}
