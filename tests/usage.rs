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
        .stdout(predicate::str::contains(
            "jag [--timeout SECONDS] [--retry-wait SECONDS] [--max-scored COUNT] TERM \"QUESTION\" [PATH...]",
        ))
        .stdout(predicate::str::contains("--max-scored <COUNT>"))
        .stdout(predicate::str::contains("--timeout <SECONDS>"))
        .stdout(predicate::str::contains("--retry-wait <SECONDS>"));
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

// FEATURE-03: the time options and the time variables.
const TIME_MESSAGE: &str = "is not a whole number of seconds from 1 to 60.";

/// A search in a folder with one match, with the made-up key.
fn time_search(extra: &[&str], environment: &[(&str, &str)]) -> std::process::Output {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        .args(extra)
        .args(["needle", "which line?"])
        .arg(dir.path())
        .output()
        .expect("run jag");
    assert!(server.requests().is_empty() || output.status.success());
    output
}

#[test]
fn a_bad_time_value_exits_2_and_no_message_shows_it() {
    let bad = "SENTINEL-SENTINEL-SENTINEL";
    type Case<'a> = (&'a [&'a str], &'a [(&'a str, &'a str)], &'a str);
    let cases: [Case; 4] = [
        (&["--timeout", bad], &[], "the value of --timeout"),
        (&["--retry-wait", bad], &[], "the value of --retry-wait"),
        (&[], &[("JAG_TIMEOUT", bad)], "JAG_TIMEOUT"),
        (&[], &[("JAG_RETRY_WAIT", bad)], "JAG_RETRY_WAIT"),
    ];
    for (extra, environment, name) in cases {
        let output = time_search(extra, environment);
        let err = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
        assert!(err.contains(name) && err.contains(TIME_MESSAGE), "{err}");
        assert!(!err.contains("SENTINEL"), "{err}");
    }
}

#[test]
fn a_time_value_out_of_range_exits_2() {
    for value in ["0", "61", "1.5", "-1", "4s"] {
        let output = time_search(&["--timeout", value], &[]);
        assert_eq!(output.status.code(), Some(2), "{value}");
        let output = time_search(&[], &[("JAG_RETRY_WAIT", value)]);
        assert_eq!(output.status.code(), Some(2), "{value}");
    }
}

#[test]
fn a_bad_time_option_comes_before_the_key_and_a_bad_variable_comes_after() {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    for option in ["--timeout", "--retry-wait"] {
        let home = home();
        jag_bare(home.path())
            .args([option, "abc", "needle", "which line?"])
            .arg(dir.path())
            .assert()
            .code(2);
    }
    for variable in ["JAG_TIMEOUT", "JAG_RETRY_WAIT"] {
        let home = home();
        jag_bare(home.path())
            .env(variable, "abc")
            .args(["needle", "which line?"])
            .arg(dir.path())
            .assert()
            .code(3);
    }
}

#[test]
fn a_time_option_with_setup_or_forget_key_gives_the_fixed_line() {
    for args in [
        ["--setup", "--timeout", "5"],
        ["--forget-key", "--retry-wait", "5"],
    ] {
        let home = home();
        jag_bare(home.path())
            .args(args)
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty())
            .stderr(FIXED_LINE);
    }
}

// FEATURE-06: the cost limit option and variable.
#[test]
fn a_bad_limit_exits_2_with_the_exact_text_and_no_value() {
    let bad = "SENTINEL-SENTINEL-SENTINEL";
    let option = time_search(&["--max-scored", bad], &[]);
    assert_eq!(option.status.code(), Some(2));
    assert!(option.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&option.stderr),
        "jag: the value of --max-scored is not a whole number from 1 to 2,000.\n\
         Use a number such as --max-scored 400.\n"
    );
    let variable = time_search(&[], &[("JAG_MAX_SCORED", bad)]);
    assert_eq!(variable.status.code(), Some(2));
    assert!(variable.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&variable.stderr),
        "jag: JAG_MAX_SCORED is not a whole number from 1 to 2,000.\n\
         Set JAG_MAX_SCORED to a number such as 400, or unset it.\n"
    );
}

#[test]
fn a_limit_out_of_range_exits_2() {
    for value in ["0", "2001", "02000", "1.5", "-1", "2,000"] {
        let output = time_search(&["--max-scored", value], &[]);
        assert_eq!(output.status.code(), Some(2), "{value}");
        let output = time_search(&[], &[("JAG_MAX_SCORED", value)]);
        assert_eq!(output.status.code(), Some(2), "{value}");
    }
}

#[test]
fn a_bad_limit_option_comes_before_the_key_and_a_bad_variable_after() {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    let home = home();
    jag_bare(home.path())
        .args(["--max-scored", "abc", "needle", "which line?"])
        .arg(dir.path())
        .assert()
        .code(2);
    jag_bare(home.path())
        .env("JAG_MAX_SCORED", "abc")
        .args(["needle", "which line?"])
        .arg(dir.path())
        .assert()
        .code(3);
}

#[test]
fn the_limit_option_with_setup_or_forget_key_gives_the_fixed_line() {
    for args in [
        ["--setup", "--max-scored", "5"],
        ["--forget-key", "--max-scored", "5"],
    ] {
        let home = home();
        jag_bare(home.path())
            .args(args)
            .assert()
            .code(2)
            .stdout(predicate::str::is_empty())
            .stderr(FIXED_LINE);
    }
}
