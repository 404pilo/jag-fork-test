//! End-to-end tests of a search with no question, and of the option `--ask`
//! (FEATURE-12, CRITERION-53, CRITERION-54, CRITERION-68, decisions 1 and 2).
//!
//! Each test sets HOME to a temporary folder. A test that proves "no model
//! call" uses the fake server and checks that it has no request.

mod common;

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;

use common::fake_server::FakeServer;
use common::{KEY, jag, jag_bare, write};
use serde_json::Value;
use tempfile::TempDir;

const USAGE_ERROR: &str = "jag: the command line is not valid. Run jag --help to see the usage.\n";
const THREE_LINES: &str =
    "dir/a.txt:1:needle one\ndir/a.txt:3:needle two\ndir/sub/b.txt:1:x needle\n";

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_of(output: Output) -> Run {
    let run = Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    assert!(!run.stdout.contains(KEY) && !run.stderr.contains(KEY));
    run
}

fn folder() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

/// The folder `dir` of the spec: two files with three matching lines.
fn example() -> TempDir {
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle one\nother\nneedle two\n");
    write(root.path(), "dir/sub/b.txt", b"x needle\n");
    root
}

/// Run `jag ARGS` in `cwd`, with the key and the fake server.
fn run(server: &FakeServer, cwd: &Path, args: &[&str]) -> Run {
    let home = folder();
    run_of(
        jag(home.path(), &server.base_url())
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("run jag"),
    )
}

/// Run `jag ARGS` in `cwd`, with no key and no setting.
fn run_bare(cwd: &Path, args: &[&str]) -> Run {
    let home = folder();
    run_of(
        jag_bare(home.path())
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("run jag"),
    )
}

#[test]
fn criterion_53_the_matching_lines_come_as_grep_gives_them() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);
    assert_eq!(
        run.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.\n"
    );
}

#[test]
fn criterion_53_the_lines_are_the_lines_of_the_grep_program() {
    let root = folder();
    write(
        root.path(),
        "one.txt",
        b"needle first\nnothing\nneedle again\n",
    );
    write(root.path(), "two.txt", b"before\nthe needle is here\n");
    write(root.path(), "sub/three.txt", b"no match\nneedle\nnothing\n");
    let jag_run = run_bare(root.path(), &["needle", "."]);
    let grep = std::process::Command::new("grep")
        .args(["-rnH", "needle", "."])
        .current_dir(root.path())
        .output()
        .expect("run grep");
    let sorted = |text: &str| {
        let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
        lines.sort();
        lines
    };
    assert_eq!(jag_run.code, Some(0));
    assert_eq!(
        sorted(&jag_run.stdout),
        sorted(&String::from_utf8_lossy(&grep.stdout))
    );
}

#[test]
fn criterion_53_one_file_shows_its_path() {
    let root = example();
    let run = run_bare(root.path(), &["needle", "dir/a.txt"]);
    assert_eq!(
        run.stdout,
        "dir/a.txt:1:needle one\ndir/a.txt:3:needle two\n"
    );
}

#[test]
fn criterion_53_a_carriage_return_stays_and_a_long_line_is_cut() {
    let root = folder();
    write(root.path(), "crlf.txt", b"needle\r\n");
    let run = run_bare(root.path(), &["needle", "crlf.txt"]);
    assert_eq!(run.stdout, "crlf.txt:1:needle\r\n");

    let long = format!("needle{}", "x".repeat(394));
    write(root.path(), "long.txt", format!("{long}\n").as_bytes());
    let run = run_bare(root.path(), &["needle", "long.txt"]);
    assert_eq!(
        run.stdout,
        format!("long.txt:1:{} [+100 chars]\n", &long[..300])
    );
}

#[test]
fn criterion_53_no_match_exits_1_and_a_bad_term_exits_2() {
    let root = example();
    let run = run_bare(root.path(), &["absent", "dir"]);
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 0 results, skipped 0.\n"
    );

    let run = run_bare(root.path(), &["(", "dir"]);
    assert_eq!(run.code, Some(2));
    assert!(run.stderr.contains("not a valid regular expression"));
}

#[test]
fn criterion_53_a_hidden_file_is_skipped_with_the_notice() {
    let root = example();
    write(root.path(), "dir/.h.txt", b"needle\n");
    let run = run_bare(root.path(), &["needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);
    assert!(
        run.stderr
            .contains("jag: skipped 1 files and folders (0 binary, 1 hidden, 0 ignored)"),
        "{}",
        run.stderr
    );
}

#[test]
fn criterion_53_the_lines_follow_the_order_of_the_paths_on_the_command_line() {
    let root = folder();
    write(root.path(), "a.txt", b"needle a\n");
    write(root.path(), "b.txt", b"needle b\n");
    let run = run_bare(root.path(), &["needle", "b.txt", "a.txt"]);
    assert_eq!(run.stdout, "b.txt:1:needle b\na.txt:1:needle a\n");
}

#[test]
fn criterion_53_a_path_that_is_given_two_times_gives_its_lines_two_times() {
    let root = folder();
    write(root.path(), "a.txt", b"needle a\n");
    write(root.path(), "b.txt", b"needle b\n");
    let run = run_bare(root.path(), &["needle", "b.txt", "a.txt", "b.txt"]);
    assert_eq!(
        run.stdout,
        "b.txt:1:needle b\na.txt:1:needle a\nb.txt:1:needle b\n"
    );
}

#[test]
fn criterion_53_paths_that_overlap_give_the_lines_at_each_place_of_the_command_line() {
    let root = folder();
    write(root.path(), "d/a.txt", b"needle a\n");
    write(root.path(), "d/sub/c.txt", b"needle c\n");
    let run = run_bare(root.path(), &["needle", "d", "d/sub", "d"]);
    assert_eq!(
        run.stdout,
        "d/a.txt:1:needle a\nd/sub/c.txt:1:needle c\n\
         d/sub/c.txt:1:needle c\n\
         d/a.txt:1:needle a\nd/sub/c.txt:1:needle c\n"
    );
}

#[test]
fn criterion_53_color_always_gives_the_bytes_of_grep() {
    let root = folder();
    write(root.path(), "a.txt", b"needle one\n");
    let run = run_bare(root.path(), &["--color=always", "needle", "a.txt"]);
    assert_eq!(
        run.stdout,
        "\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[32m\x1b[K1\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K\x1b[01;31m\x1b[Kneedle\x1b[m\x1b[K one\n"
    );
}

#[test]
fn criterion_54_no_model_call_is_made() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_54_the_document_has_the_lines_and_no_score() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--json", "needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, "");
    let document: Value = serde_json::from_str(&run.stdout).expect("one document");
    assert_eq!(document["exit_code"], 0);
    assert_eq!(document["error"], Value::Null);
    assert_eq!(document["no_good_answer"], false);
    assert_eq!(document["results_found"], 3);
    assert_eq!(document["results_scored"], 0);
    assert_eq!(document["results_skipped"], 0);
    assert_eq!(document["prompt_version"], "jag-p1");
    let results = document["results"].as_array().expect("a list");
    assert_eq!(results.len(), 3);
    let paths = ["dir/a.txt", "dir/a.txt", "dir/sub/b.txt"];
    for (index, result) in results.iter().enumerate() {
        assert_eq!(result["rank"], index + 1);
        assert_eq!(result["path"], paths[index]);
        assert_eq!(result["first_line"], result["last_line"]);
        assert_eq!(result["lines"].as_array().expect("lines").len(), 1);
        assert_eq!(result["lines"][0]["is_match"], true);
    }
    assert_eq!(results[1]["first_line"], 3);
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_54_a_search_with_no_line_gives_the_document_with_exit_1() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--json", "absent", "dir"]);
    assert_eq!(run.code, Some(1));
    let document: Value = serde_json::from_str(&run.stdout).expect("one document");
    assert_eq!(document["exit_code"], 1);
    assert_eq!(document["results"], serde_json::json!([]));
}

#[test]
fn criterion_54_the_options_of_the_model_have_no_effect() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["--max-scored", "1", "needle", "dir"],
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);
    assert_eq!(
        run.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.\n"
    );

    let run = self::run(
        &server,
        root.path(),
        &["--timeout", "60", "--retry-wait", "60", "needle", "dir"],
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);

    let run = run_bare(root.path(), &["--timeout", "abc", "needle", "dir"]);
    assert_eq!(run.code, Some(2));
    assert!(
        run.stderr.contains("the value of --timeout"),
        "{}",
        run.stderr
    );
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_68_no_key_is_needed_and_none_is_asked_for() {
    let root = example();
    let run = run_bare(root.path(), &["needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, THREE_LINES);
    assert_eq!(
        run.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.\n"
    );

    let run = run_bare(root.path(), &["--json", "needle", "dir"]);
    assert_eq!(run.code, Some(0));
    let document: Value = serde_json::from_str(&run.stdout).expect("one document");
    assert_eq!(document["results"].as_array().expect("a list").len(), 3);
}

#[test]
fn criterion_68_an_open_key_file_stops_a_question_search_and_not_plain_grep() {
    let root = example();
    let home = folder();
    let config = home.path().join(".config/jag");
    std::fs::create_dir_all(&config).expect("make folder");
    let file = config.join("config.yml");
    std::fs::write(&file, "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\n").expect("write");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("mode");

    let plain = run_of(
        jag_bare(home.path())
            .current_dir(root.path())
            .args(["needle", "dir"])
            .output()
            .expect("run jag"),
    );
    assert_eq!(plain.code, Some(0));
    assert_eq!(plain.stdout, THREE_LINES);

    let asked = run_of(
        jag_bare(home.path())
            .current_dir(root.path())
            .args(["--ask", "which line?", "needle", "dir"])
            .output()
            .expect("run jag"),
    );
    assert_eq!(asked.code, Some(3));
    assert!(asked.stderr.contains("can be read by other users"));
}

#[test]
fn criterion_68_the_variables_of_the_model_are_not_read() {
    let root = example();
    for (name, value) in [
        ("JAG_BASE_URL", "http://example.com/v1"),
        ("JAG_TIMEOUT", "abc"),
        ("JAG_RETRY_WAIT", "abc"),
        ("JAG_MAX_SCORED", "abc"),
        ("JAG_MODEL", ""),
    ] {
        let home = folder();
        let run = run_of(
            jag_bare(home.path())
                .env(name, value)
                .current_dir(root.path())
                .args(["needle", "dir"])
                .output()
                .expect("run jag"),
        );
        assert_eq!(run.code, Some(0), "{name}");
        assert_eq!(run.stdout, THREE_LINES, "{name}");
    }
}

#[test]
fn decision_2_the_option_ask_gives_the_question_in_each_place() {
    let root = example();
    for args in [
        &["--ask", "which line?", "needle", "dir"][..],
        &["--ask=which line?", "needle", "dir"],
        &["needle", "dir", "--ask", "which line?"],
    ] {
        let server = FakeServer::constant(0.9);
        let run = run(&server, root.path(), args);
        assert_eq!(run.code, Some(0), "{args:?}");
        let requests = server.requests();
        assert_eq!(requests.len(), 1, "{args:?}");
        assert_eq!(requests[0].json()["state"]["question"], "which line?");
    }
}

#[test]
fn decision_1_a_word_after_the_term_is_a_path() {
    let root = folder();
    write(root.path(), "my notes/a.txt", b"needle\n");
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["needle", "my notes"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "my notes/a.txt:1:needle\n");
    assert!(server.requests().is_empty());
}

#[test]
fn decision_1_the_old_form_makes_no_call_and_names_the_path() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["needle", "which line says retry", "dir"],
    );
    assert!(server.requests().is_empty());
    assert_eq!(run.stdout, THREE_LINES);
    // FEATURE-14, decisions 7 and 20: the warning of a path that does not
    // exist, and exit 2 at the end.
    assert_eq!(run.code, Some(2));
    assert!(
        run.stderr
            .contains("jag: the path which line says retry does not exist"),
        "{}",
        run.stderr
    );
}

#[test]
fn decision_2_after_a_double_dash_ask_is_a_term() {
    let root = folder();
    write(root.path(), "dir/a.txt", b"use --ask here\n");
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--", "--ask", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "dir/a.txt:1:use --ask here\n");
    assert!(server.requests().is_empty());
}

#[test]
fn decision_2_a_question_that_starts_with_a_dash_needs_an_equals_sign() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--ask=-v", "needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(server.requests().len(), 1);
    assert_eq!(server.requests()[0].json()["state"]["question"], "-v");

    let server = FakeServer::constant(0.9);
    let run = self::run(&server, root.path(), &["--ask", "-v", "needle", "dir"]);
    assert_eq!(run.code, Some(2));
    // `-v` is an unknown option, and jag names it (FEATURE-13, decision 18).
    assert_eq!(
        run.stderr,
        "jag: the option -v is not valid.\nRun jag --help to see the options.\n"
    );
    assert!(server.requests().is_empty());
}

#[test]
fn decision_2_a_json_option_is_never_the_value_of_ask() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--ask", "--json", "needle", "dir"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stderr, "");
    let document: Value = serde_json::from_str(&run.stdout).expect("one document");
    assert_eq!(document["error"]["cause"], "the command line is not valid.");
    assert!(server.requests().is_empty());
}

#[test]
fn decision_2_bad_forms_of_ask_are_argument_errors() {
    let root = example();
    let server = FakeServer::constant(0.9);
    for args in [
        &["--ask", "a", "--ask", "b", "needle", "dir"][..],
        &["needle", "dir", "--ask"],
        &["--check", "--ask", "q"],
        &["--setup", "--ask", "q"],
        &["--forget-key", "--ask", "q"],
    ] {
        let run = run(&server, root.path(), args);
        assert_eq!(run.code, Some(2), "{args:?}");
        assert_eq!(run.stderr, USAGE_ERROR, "{args:?}");
    }
    assert!(server.requests().is_empty());
}

#[test]
fn decision_3_a_question_that_is_not_utf8_is_an_argument_error() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let home = folder();
    let output = jag(home.path(), &server.base_url())
        .current_dir(root.path())
        .arg("--ask")
        .arg(OsStr::from_bytes(b"how \xff"))
        .args(["needle", "dir"])
        .output()
        .expect("run jag");
    let run = run_of(output);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stderr, USAGE_ERROR);
    assert!(server.requests().is_empty());
}

#[test]
fn decision_20_ask_with_no_term_says_that_a_term_is_needed() {
    let root = example();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--ask", "q"]);
    assert_eq!(run.code, Some(2));
    assert!(
        run.stderr.contains("a search needs a term."),
        "{}",
        run.stderr
    );
    assert!(server.requests().is_empty());
}
