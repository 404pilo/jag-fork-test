//! Tests of the argument errors: no message repeats a value from the command
//! line, because a value can be a key (CV-6, decision 26).

mod common;

use common::fake_server::FakeServer;
use common::{jag, jag_bare, write};
use predicates::prelude::*;

const FIXED_LINE: &str = "jag: the command line is not valid. Run jag --help to see the usage.\n";

fn home() -> tempfile::TempDir {
    tempfile::tempdir().expect("temporary folder")
}

#[test]
fn an_argument_error_writes_one_fixed_line_with_no_value() {
    let cases: [&[&str]; 5] = [
        &["--setup", "SENTINEL-SENTINEL-SENTINEL"],
        &["--setup=SENTINEL-SENTINEL-SENTINEL"],
        &["--SENTINEL-SENTINEL-SENTINEL"],
        &["--forget-key", "needle", "SENTINEL-SENTINEL-SENTINEL"],
        &["--forget-key=SENTINEL-SENTINEL-SENTINEL"],
    ];
    for args in cases {
        let home = home();
        jag_bare(home.path())
            .args(args)
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty())
            .stderr(FIXED_LINE)
            .stderr(predicate::str::contains("SENTINEL").not());
    }
}

#[test]
fn help_and_version_exit_0() {
    let home = home();
    jag_bare(home.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("jag TERM \"QUESTION\" [PATH...]"));
    jag_bare(home.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jag"));
}

#[test]
fn no_argument_shows_the_help_and_exits_2() {
    let home = home();
    jag_bare(home.path())
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("TERM"));
}

#[test]
fn a_term_with_no_question_exits_2_and_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    jag(home.path(), &server.base_url())
        .arg("needle")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("needs a question"))
        .stderr(predicate::str::contains("needle").not());
    assert!(server.requests().is_empty());
}

#[test]
fn a_term_and_a_question_with_no_path_exit_2_and_make_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    jag(home.path(), &server.base_url())
        .args(["needle", "which line?"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("needs a path"))
        .stderr(predicate::str::contains("which line?").not());
    assert!(server.requests().is_empty());
}

#[test]
fn a_usage_error_comes_before_the_key_lookup() {
    // No key at all, and still exit 2 for a missing question.
    let home = home();
    jag_bare(home.path()).arg("needle").assert().code(2);
}

#[test]
fn a_blank_question_exits_2() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    jag(home.path(), &server.base_url())
        .args(["needle", "   "])
        .arg(dir.path())
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty());
    assert!(server.requests().is_empty());
}

#[test]
fn the_question_limit_is_12000_bytes() {
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");

    let server = FakeServer::constant(0.8);
    jag(home.path(), &server.base_url())
        .args(["needle", &"a".repeat(12_000)])
        .arg(dir.path())
        .assert()
        .success();
    assert_eq!(server.requests().len(), 1);

    for question in ["a".repeat(12_001), "漢".repeat(4_001)] {
        let server = FakeServer::constant(0.8);
        jag(home.path(), &server.base_url())
            .args(["needle"])
            .arg(&question)
            .arg(dir.path())
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains("aaaa").not())
            .stderr(predicate::str::contains("漢").not());
        assert!(server.requests().is_empty());
    }
}

#[test]
fn a_bad_regular_expression_exits_2_and_the_term_is_not_repeated() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    jag(home.path(), &server.base_url())
        .args(["SENTINEL-SENTINEL-SENTINEL(", "which line?"])
        .arg(dir.path())
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("not a valid regular expression"))
        .stderr(predicate::str::contains("SENTINEL").not());
    assert!(server.requests().is_empty());
}

#[test]
fn a_base_url_that_is_not_safe_exits_2_and_is_not_repeated() {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    for bad in [
        "http://example.com/v1",
        "http://localhost.example.net/v1",
        "http://127.0.0.1@example.net/v1",
        "https://user@example.com/v1",
    ] {
        let home = home();
        jag(home.path(), bad)
            .args(["needle", "which line?"])
            .arg(dir.path())
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains("JAG_BASE_URL"))
            .stderr(predicate::str::contains("example").not())
            .stderr(predicate::str::contains("user").not());
    }
}

#[test]
fn a_slash_at_the_end_of_the_base_url_is_removed() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    jag(home.path(), &format!("{}/", server.base_url()))
        .args(["needle", "which line?"])
        .arg(dir.path())
        .assert()
        .success();
    assert_eq!(server.requests()[0].path, "/v1/systemone");
}

#[test]
fn localhost_is_an_allowed_plain_http_host() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    // The address passes the check, so the exit code is not 2.
    let output = jag(
        home.path(),
        &format!("http://localhost:{}/v1", server.port()),
    )
    .args(["needle", "which line?"])
    .arg(dir.path())
    .output()
    .expect("run jag");
    assert_ne!(output.status.code(), Some(2));
}
