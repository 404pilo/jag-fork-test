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
        .stdout(predicate::str::contains("jag [OPTIONS] TERM [PATH...]"))
        .stdout(predicate::str::contains(
            "jag [OPTIONS] --ask QUESTION TERM [PATH...]",
        ))
        .stdout(predicate::str::contains("--ask <QUESTION>"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--color[=<WHEN>]"))
        .stdout(predicate::str::contains("--max-scored <COUNT>"))
        .stdout(predicate::str::contains("--max-chars <COUNT>"))
        .stdout(predicate::str::contains("--no-max-chars"))
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
fn a_term_with_no_path_exits_2_and_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    jag(home.path(), &server.base_url())
        .arg("needle")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("needs a path"))
        .stderr(predicate::str::contains("needle").not());
    assert!(server.requests().is_empty());
}

#[test]
fn a_term_and_a_question_with_no_path_exit_2_and_make_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    jag(home.path(), &server.base_url())
        .args(["--ask", "which line?", "needle"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("needs a path"))
        .stderr(predicate::str::contains("which line?").not());
    assert!(server.requests().is_empty());
}

#[test]
fn a_usage_error_comes_before_the_key_lookup() {
    // No key at all, and still exit 2 for a blank question.
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    jag_bare(home.path())
        .args(["--ask", "   ", "needle"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn a_blank_question_exits_2() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    jag(home.path(), &server.base_url())
        .args(["--ask", "   ", "needle"])
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
        .args(["--ask", &"a".repeat(12_000), "needle"])
        .arg(dir.path())
        .assert()
        .success();
    assert_eq!(server.requests().len(), 1);

    for question in ["a".repeat(12_001), "漢".repeat(4_001)] {
        let server = FakeServer::constant(0.8);
        jag(home.path(), &server.base_url())
            .arg("--ask")
            .arg(&question)
            .arg("needle")
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
        .args(["--ask", "which line?", "SENTINEL-SENTINEL-SENTINEL("])
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
            .args(["--ask", "which line?", "needle"])
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
        .args(["--ask", "which line?", "needle"])
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
    .args(["--ask", "which line?", "needle"])
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
        .args(["--ask", "which line?", "needle"])
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
            .args([option, "abc", "--ask", "which line?", "needle"])
            .arg(dir.path())
            .assert()
            .code(2);
    }
    for variable in ["JAG_TIMEOUT", "JAG_RETRY_WAIT"] {
        let home = home();
        jag_bare(home.path())
            .env(variable, "abc")
            .args(["--ask", "which line?", "needle"])
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
        .args(["--max-scored", "abc", "--ask", "which line?", "needle"])
        .arg(dir.path())
        .assert()
        .code(2);
    jag_bare(home.path())
        .env("JAG_MAX_SCORED", "abc")
        .args(["--ask", "which line?", "needle"])
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

// FEATURE-09, CRITERION-43: the size limit option and variable.
const OPTION_TEXT: &str = "jag: the value of --max-chars is not a whole number of 1,000 or more.\n\
     Use a number such as --max-chars 40000, or turn the limit off with --no-max-chars.\n";
const VARIABLE_TEXT: &str = "jag: JAG_MAX_CHARS is not a whole number of 1,000 or more.\n\
     Set JAG_MAX_CHARS to a number such as 40000, or unset it.\n";

const BAD_SIZES: [&str; 10] = [
    "0", "999", "0999", "abc", "1.5", "1e4", "+1000", "20,000", "off", " ",
];

#[test]
fn criterion_43_a_size_limit_that_is_not_valid_exits_2_with_the_exact_text() {
    for value in BAD_SIZES {
        let output = time_search(&["--max-chars", value], &[]);
        assert_eq!(output.status.code(), Some(2), "{value:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&output.stderr), OPTION_TEXT);
        let output = time_search(&[], &[("JAG_MAX_CHARS", value)]);
        // A blank variable is the default.
        if value.trim().is_empty() {
            assert_eq!(output.status.code(), Some(0));
            continue;
        }
        assert_eq!(output.status.code(), Some(2), "{value:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8_lossy(&output.stderr), VARIABLE_TEXT);
    }
}

#[test]
fn criterion_43_a_negative_limit_exits_2() {
    let output = time_search(&["--max-chars=-1"], &[]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stderr), OPTION_TEXT);
    let output = time_search(&["--max-chars", "-1"], &[]);
    assert_eq!(output.status.code(), Some(2));
    let output = time_search(&[], &[("JAG_MAX_CHARS", "-1")]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stderr), VARIABLE_TEXT);
}

#[test]
fn a_size_limit_of_1000_or_more_is_valid() {
    for value in ["1000", "01000", " 40000 ", "99999999999999999999999"] {
        let output = time_search(&["--max-chars", value], &[]);
        assert_eq!(output.status.code(), Some(0), "{value:?}");
        let output = time_search(&[], &[("JAG_MAX_CHARS", value)]);
        assert_eq!(output.status.code(), Some(0), "{value:?}");
    }
    assert_eq!(time_search(&["--no-max-chars"], &[]).status.code(), Some(0));
}

#[test]
fn a_bad_size_limit_never_shows_its_value() {
    let bad = "SENTINEL-SENTINEL-SENTINEL";
    let option = time_search(&["--max-chars", bad], &[]);
    assert_eq!(option.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&option.stderr), OPTION_TEXT);
    let variable = time_search(&[], &[("JAG_MAX_CHARS", bad)]);
    assert_eq!(variable.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&variable.stderr), VARIABLE_TEXT);
}

#[test]
fn a_bad_size_option_comes_before_the_key_and_a_bad_variable_after() {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    let home = home();
    jag_bare(home.path())
        .args(["--max-chars", "abc", "--ask", "which line?", "needle"])
        .arg(dir.path())
        .assert()
        .code(2);
    jag_bare(home.path())
        .env("JAG_MAX_CHARS", "abc")
        .args(["--ask", "which line?", "needle"])
        .arg(dir.path())
        .assert()
        .code(3);
    // With no question, jag reads no key, so the variable gives exit 2.
    jag_bare(home.path())
        .env("JAG_MAX_CHARS", "abc")
        .args(["needle"])
        .arg(dir.path())
        .assert()
        .code(2);
}

#[test]
fn the_size_options_conflict_with_each_other_and_with_the_other_modes() {
    for args in [
        &["--max-chars", "5000", "--no-max-chars"][..],
        &["--check", "--max-chars", "5000"],
        &["--check", "--no-max-chars"],
        &["--setup", "--max-chars", "5000"],
        &["--forget-key", "--no-max-chars"],
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

// FEATURE-11, CRITERION-51: with `--json`, an argument error is a document on
// standard output, and it holds no value from the command line.
#[test]
fn an_argument_error_with_json_writes_one_document_with_no_value() {
    let cases: [&[&str]; 6] = [
        &["--json", "--setup"],
        &["--json", "--forget-key"],
        &["--json", "--check"],
        &["--json", "--bogus"],
        &["--json", "--timeout"],
        &["--json", "--setup", "SENTINEL-SENTINEL-SENTINEL"],
    ];
    for args in cases {
        let home = home();
        let output = jag_bare(home.path()).args(args).output().expect("run jag");
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.ends_with('\n') && text.matches('\n').count() == 1);
        let document: serde_json::Value = serde_json::from_str(&text).expect("a document");
        assert_eq!(document["exit_code"], 2);
        assert_eq!(document["error"]["cause"], "the command line is not valid.");
        assert_eq!(document["error"]["fix"], "Run jag --help to see the usage.");
        assert_eq!(document["results"], serde_json::json!([]));
        assert!(!text.contains("bogus"));
        assert!(!text.contains("SENTINEL"));
    }
}

#[test]
fn json_after_a_double_dash_is_not_the_option() {
    let home = home();
    jag_bare(home.path())
        .args(["--bogus", "--", "--json"])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(FIXED_LINE);
}

#[test]
fn help_and_version_with_json_write_their_text() {
    let home = home();
    jag_bare(home.path())
        .args(["--json", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("\"version\"").not());
    jag_bare(home.path())
        .args(["--json", "--version"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("jag "));
}

#[test]
fn json_with_no_term_gives_an_error_document() {
    let home = home();
    let output = jag_bare(home.path())
        .arg("--json")
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).expect("a document");
    assert_eq!(document["error"]["cause"], "a search needs a term.");
    assert_eq!(document["exit_code"], 2);
}
