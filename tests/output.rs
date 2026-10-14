//! End-to-end tests of the text and the structured output (FEATURE-11,
//! CRITERION-44 to CRITERION-52), against the fake server.
//!
//! Each test sets HOME to a temporary folder, uses the made-up key of
//! `common::KEY`, and calls only the fake server on 127.0.0.1.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::fake_server::{FakeServer, Reply};
use common::{KEY, jag, jag_bare, write};
use serde_json::Value;
use tempfile::TempDir;

const QUESTION: &str = "which line says how to retry";

fn folder() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn document(&self) -> Value {
        assert!(self.stdout.ends_with('\n'), "{}", self.stdout);
        assert_eq!(self.stdout.matches('\n').count(), 1, "{}", self.stdout);
        serde_json::from_str(&self.stdout).expect("one JSON document")
    }
}

/// Run `jag --ask QUESTION needle PATH...` from the folder `cwd`, with the key in
/// a variable. No output holds the key.
fn run(
    server: &FakeServer,
    cwd: &Path,
    options: &[&str],
    environment: &[(&str, &str)],
    paths: &[&str],
) -> Run {
    let home = folder();
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        .current_dir(cwd)
        .args(options)
        .args(["--ask", QUESTION, "needle"])
        .args(paths)
        .output()
        .expect("run jag");
    finish(output)
}

fn finish(output: std::process::Output) -> Run {
    let run = Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    assert!(!run.stdout.contains(KEY));
    assert!(!run.stderr.contains(KEY));
    run
}

/// Two files with one match each, in the folder `dir`.
fn two_results() -> TempDir {
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle a\n");
    write(root.path(), "dir/b.txt", b"needle b\n");
    root
}

const TWO_RESULTS_TEXT: &str = "dir/a.txt:1:needle a\n--\ndir/b.txt:1:needle b\n";

/// Hidden, binary, and locked files, and two results (CRITERION-45). Gives
/// the folder and whether the locked file can be read (root).
fn notice_folder() -> (TempDir, bool) {
    let root = folder();
    write(root.path(), "dir/.hidden.txt", b"needle\n");
    write(root.path(), "dir/binary.dat", b"\0needle\n");
    write(root.path(), "dir/locked.txt", b"needle\n");
    write(root.path(), "dir/a.txt", b"needle a\n");
    write(root.path(), "dir/b.txt", b"needle b\n");
    let locked = root.path().join("dir/locked.txt");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("lock");
    let can_read = std::fs::read(&locked).is_ok();
    (root, can_read)
}

const LIMIT_ONE: &[(&str, &str)] = &[("JAG_MAX_SCORED", "1")];

// CRITERION-44: text is the default, also in a pipe.
#[test]
fn criterion_44_text_is_the_default_also_in_a_pipe() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let run = run(&server, root.path(), &[], &[], &["dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, TWO_RESULTS_TEXT);
    assert!(serde_json::from_str::<Value>(&run.stdout).is_err());
    assert_eq!(run.stderr, "");
}

#[test]
fn criterion_44_no_variable_turns_on_structured_output() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let environment = [
        ("JAG_JSON", "1"),
        ("JAG_OUTPUT", "json"),
        ("JAG_FORMAT", "json"),
    ];
    let run = run(&server, root.path(), &[], &environment, &["dir"]);
    assert_eq!(run.stdout, TWO_RESULTS_TEXT);
}

// CRITERION-45: the results on standard output, each notice on the error
// stream.
#[test]
fn criterion_45_text_mode_has_the_results_on_stdout_and_each_notice_on_stderr() {
    let server = FakeServer::constant(0.8);
    let (root, can_read) = notice_folder();
    let run = run(&server, root.path(), &[], LIMIT_ONE, &["dir"]);
    unlock(root.path());
    assert_eq!(run.code, Some(7));
    for line in run.stdout.lines() {
        assert!(line.starts_with("dir/") || line == "--", "{line}");
    }
    assert!(!run.stdout.is_empty());
    for line in run.stderr.lines() {
        assert!(line.starts_with("jag: "), "{line}");
    }
    assert!(run.stderr.contains("jag: skipped 2 files and folders"));
    assert!(run.stderr.contains("jag: cost limit reached"));
    // A user who reads any file (root) gets no warning.
    if !can_read {
        assert!(run.stderr.contains("jag: cannot read dir/locked.txt"));
    }
}

fn unlock(root: &Path) {
    let locked = root.join("dir/locked.txt");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).expect("unlock");
}

// CRITERION-46: only the option turns on structured output.
#[test]
fn criterion_46_the_option_gives_one_line_with_one_object() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let run = run(&server, root.path(), &["--json"], &[], &["dir"]);
    assert_eq!(run.code, Some(0));
    assert!(run.document().is_object());
    assert_eq!(run.stderr, "");
}

/// Build the text of the CRITERION-47 table from the fields of the document.
fn text_from(document: &Value) -> String {
    let mut text = String::new();
    if document["no_good_answer"] == true {
        text.push_str("jag: no good answer. The results below do not answer the question well.\n");
    }
    for (index, result) in document["results"]
        .as_array()
        .expect("results")
        .iter()
        .enumerate()
    {
        if index > 0 {
            text.push_str("--\n");
        }
        for line in result["lines"].as_array().expect("lines") {
            let separator = if line["is_match"] == true { ':' } else { '-' };
            text.push_str(&format!(
                "{}{separator}{}{separator}{}",
                result["path"].as_str().expect("path"),
                line["number"],
                line["text"].as_str().expect("text"),
            ));
            let cut = line["cut"].as_u64().expect("cut");
            if cut > 0 {
                text.push_str(&format!(" [+{cut} chars]"));
            }
            text.push('\n');
        }
    }
    text
}

// CRITERION-47: the same results, and a named field for each item.
#[test]
fn criterion_47_the_document_holds_the_same_results_as_the_text() {
    let server = FakeServer::constant(0.8);
    let root = folder();
    let long = format!("needle {}", "x".repeat(400));
    let text = format!("ctx one\nctx two\n{long}\nctx three\nctx four\n");
    write(root.path(), "dir/a.txt", text.as_bytes());
    write(root.path(), "dir/b.txt", b"needle b\n");
    let in_text = run(&server, root.path(), &[], &[], &["dir"]);
    let in_json = run(&server, root.path(), &["--json"], &[], &["dir"]);
    assert_eq!(in_json.code, in_text.code);
    assert!(in_text.stdout.contains(" [+"));
    assert!(in_text.stdout.contains("dir/a.txt-1-ctx one\n"));
    assert!(in_text.stdout.contains("dir/a.txt:3:needle "));
    assert_eq!(text_from(&in_json.document()), in_text.stdout);
}

#[test]
fn criterion_47_a_low_score_is_the_label_in_text_and_a_field_in_json() {
    let server = FakeServer::constant(0.2);
    let root = two_results();
    let in_text = run(&server, root.path(), &[], &[], &["dir"]);
    let in_json = run(&server, root.path(), &["--json"], &[], &["dir"]);
    assert_eq!(in_text.code, Some(1));
    assert_eq!(in_json.code, Some(1));
    assert!(in_text.stdout.starts_with("jag: no good answer."));
    let document = in_json.document();
    assert_eq!(document["no_good_answer"], true);
    assert_eq!(document["exit_code"], 1);
    assert_eq!(text_from(&document), in_text.stdout);

    let server = FakeServer::constant(0.8);
    let document = run(&server, root.path(), &["--json"], &[], &["dir"]).document();
    assert_eq!(document["no_good_answer"], false);
}

/// The three numbers of the skip notice and the three of the limit notice
/// from the error stream of a text run.
fn numbers(text: &str) -> Vec<u64> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse().expect("number"))
        .collect()
}

// CRITERION-47 and CRITERION-50: the notices are fields, and the error stream
// is empty.
#[test]
fn criterion_50_each_notice_is_a_field_and_the_error_stream_is_empty() {
    let server = FakeServer::constant(0.8);
    let (root, can_read) = notice_folder();
    let in_text = run(&server, root.path(), &[], LIMIT_ONE, &["dir"]);
    let in_json = run(&server, root.path(), &["--json"], LIMIT_ONE, &["dir"]);
    unlock(root.path());
    assert_eq!(in_json.code, Some(7));
    assert_eq!(in_json.stderr, "");
    let document = in_json.document();

    let skip = in_text
        .stderr
        .lines()
        .find(|line| line.starts_with("jag: skipped"))
        .expect("the skip notice");
    let skip = numbers(skip);
    let files = &document["files_skipped"];
    assert_eq!(
        [
            &files["total"],
            &files["binary"],
            &files["hidden"],
            &files["ignored"]
        ],
        [skip[0], skip[1], skip[2], skip[3]]
    );

    let limit = in_text
        .stderr
        .lines()
        .find(|line| line.starts_with("jag: cost limit reached"))
        .expect("the limit notice");
    let limit = numbers(limit);
    // "scored K of N results, skipped S."
    assert_eq!(document["results_scored"], limit[0]);
    assert_eq!(document["results_found"], limit[1]);
    assert_eq!(document["results_skipped"], limit[2]);
    // A user who reads any file (root) also finds the locked file.
    let found = if can_read { 3 } else { 2 };
    assert_eq!(document["results_found"], found);
    assert_eq!(document["results_scored"], 1);
    assert_eq!(document["results_skipped"], found - 1);

    // A user who reads any file (root) gets no warning.
    if can_read {
        assert_eq!(document["warnings"], serde_json::json!([]));
        return;
    }
    let warning_line = in_text
        .stderr
        .lines()
        .find(|line| line.starts_with("jag: cannot read"))
        .expect("the warning");
    let warnings = document["warnings"].as_array().expect("warnings");
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0]["text"].as_str(),
        warning_line.strip_prefix("jag: ")
    );
    assert_eq!(warnings[0]["path"], "dir/locked.txt");
    assert_eq!(document["results"].as_array().map(Vec::len), Some(1));
}

// CRITERION-48: a rank and no score.
#[test]
fn criterion_48_each_result_has_a_rank_and_no_score() {
    let server = FakeServer::start(|request| {
        Reply::scores(request, |text| {
            if text.contains("top") {
                0.9
            } else if text.contains("mid") {
                0.7
            } else {
                0.5
            }
        })
    });
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle low\n");
    write(root.path(), "dir/b.txt", b"needle top\n");
    write(root.path(), "dir/c.txt", b"needle mid\n");
    let in_text = run(&server, root.path(), &[], &[], &["dir"]);
    let in_json = run(&server, root.path(), &["--json"], &[], &["dir"]);
    let document = in_json.document();
    let results = document["results"].as_array().expect("results");
    let paths: Vec<&str> = results
        .iter()
        .map(|result| result["path"].as_str().expect("path"))
        .collect();
    assert_eq!(paths, ["dir/b.txt", "dir/c.txt", "dir/a.txt"]);
    let ranks: Vec<u64> = results
        .iter()
        .map(|result| result["rank"].as_u64().expect("rank"))
        .collect();
    assert_eq!(ranks, [1, 2, 3]);
    assert_eq!(text_from(&document), in_text.stdout);
    assert_no_score_and_whole_numbers(&document);
}

fn assert_no_score_and_whole_numbers(value: &Value) {
    match value {
        Value::Number(number) => assert!(number.is_u64(), "{number}"),
        Value::Array(list) => list.iter().for_each(assert_no_score_and_whole_numbers),
        Value::Object(map) => {
            assert!(!map.contains_key("score"));
            map.values().for_each(assert_no_score_and_whole_numbers);
        }
        _ => {}
    }
}

const TOP_LEVEL: &[&str] = &[
    "version",
    "prompt_version",
    "exit_code",
    "error",
    "no_good_answer",
    "results_found",
    "results_scored",
    "results_skipped",
    "left_out",
    "top_result_cut",
    "size_limit_fix",
    "files_skipped",
    "warnings",
    "warnings_left_out",
    "results",
];

fn assert_top_level(document: &Value) {
    let mut names: Vec<&str> = document
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    names.sort_unstable();
    let mut want = TOP_LEVEL.to_vec();
    want.sort_unstable();
    assert_eq!(names, want);
}

// CRITERION-49: a complete document with no result.
#[test]
fn criterion_49_no_match_gives_a_complete_document_and_exit_1() {
    let server = FakeServer::constant(0.8);
    let root = folder();
    write(root.path(), "dir/a.txt", b"nothing here\n");
    let in_json = run(&server, root.path(), &["--json"], &[], &["dir"]);
    assert_eq!(in_json.code, Some(1));
    assert_eq!(in_json.stderr, "");
    let document = in_json.document();
    assert_top_level(&document);
    assert_eq!(document["results"], serde_json::json!([]));
    assert_eq!(document["exit_code"], 1);
    assert!(document["error"].is_null());
    assert_eq!(document["results_found"], 0);
    assert!(server.requests().is_empty());

    let in_text = run(&server, root.path(), &[], &[], &["dir"]);
    assert_eq!(in_text.code, Some(1));
    assert_eq!(in_text.stdout, "");
}

/// The cause and the fix of the text mode: the first line with no `jag: `,
/// and the other lines.
fn failure_of(stderr: &str) -> (String, String) {
    let text = stderr.strip_prefix("jag: ").expect("a failure");
    let (cause, fix) = text.split_once('\n').expect("a fix");
    (cause.to_owned(), fix.trim_end_matches('\n').to_owned())
}

fn assert_error_document(run: &Run, code: i32, text_stderr: &str) {
    assert_eq!(run.code, Some(code));
    assert_eq!(run.stderr, "");
    let document = run.document();
    assert_top_level(&document);
    assert_eq!(document["exit_code"], code);
    let (cause, fix) = failure_of(text_stderr);
    assert_eq!(document["error"]["cause"], cause.as_str());
    assert_eq!(document["error"]["fix"], fix.as_str());
    assert_eq!(document["results"], serde_json::json!([]));
}

// CRITERION-51: each error is a document with the cause, the fix, and the code.
#[test]
fn criterion_51_an_invalid_term_gives_an_error_document_with_exit_2() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let home = folder();
    let text = jag(home.path(), &server.base_url())
        .current_dir(root.path())
        .args(["--ask", QUESTION, "(", "dir"])
        .output()
        .expect("run jag");
    let text = finish(text);
    let json = jag(home.path(), &server.base_url())
        .current_dir(root.path())
        .args(["--json", "--ask", QUESTION, "(", "dir"])
        .output()
        .expect("run jag");
    assert_error_document(&finish(json), 2, &text.stderr);
}

#[test]
fn criterion_51_no_key_gives_an_error_document_with_exit_3() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let home = folder();
    let command = |options: &[&str]| {
        finish(
            jag_bare(home.path())
                .env("JAG_BASE_URL", server.base_url())
                .current_dir(root.path())
                .args(options)
                .args(["--ask", QUESTION, "needle", "dir"])
                .output()
                .expect("run jag"),
        )
    };
    let text = command(&[]);
    assert_error_document(&command(&["--json"]), 3, &text.stderr);
    assert!(server.requests().is_empty());
}

fn failing(status: u16) -> FakeServer {
    FakeServer::start(move |_| Reply::status(status))
}

#[test]
fn criterion_51_http_401_402_and_500_give_error_documents() {
    let root = two_results();
    for (status, code) in [(401, 4), (402, 5), (500, 6)] {
        let server = failing(status);
        let options = ["--retry-wait", "1"];
        let text = run(&server, root.path(), &options, &[], &["dir"]);
        let json = run(
            &server,
            root.path(),
            &["--json", "--retry-wait", "1"],
            &[],
            &["dir"],
        );
        assert_error_document(&json, code, &text.stderr);
        // The scan found the results before the call failed.
        let document = json.document();
        assert_eq!(document["results_found"], 2, "{status}");
        assert_eq!(document["results_scored"], 2, "{status}");
    }
}

#[test]
fn criterion_51_a_failure_keeps_the_counts_and_no_result() {
    let server = failing(500);
    let root = two_results();
    let json = run(
        &server,
        root.path(),
        &["--json", "--retry-wait", "1"],
        LIMIT_ONE,
        &["dir"],
    );
    assert_eq!(json.code, Some(6));
    let document = json.document();
    assert_eq!(document["results_found"], 2);
    assert_eq!(document["results_scored"], 1);
    assert_eq!(document["results_skipped"], 1);
    assert_eq!(document["results"], serde_json::json!([]));
    assert_eq!(json.stderr, "");
}

// A write that fails goes to the error stream as text, and not as a second
// document (decision 25).
#[cfg(target_os = "linux")]
#[test]
fn criterion_51_a_document_that_cannot_be_written_gives_the_text_failure_and_exit_2() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let home = folder();
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    // A plain `std` command, because `assert_cmd` has no way to set the
    // standard output.
    let output = std::process::Command::new(assert_cmd::cargo::cargo_bin!("jag"))
        .env_clear()
        .env("HOME", home.path())
        .env("JAG_TYPESAFE_API_KEY", KEY)
        .env("JAG_BASE_URL", server.base_url())
        .current_dir(root.path())
        .args(["--json", "--ask", QUESTION, "needle", "dir"])
        .stdout(std::process::Stdio::from(full))
        .output()
        .expect("run jag");
    let run = finish(output);
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        "jag: jag could not write the results.\nCheck where the output goes.\n"
    );
}

// An argument error document that cannot be written also goes to the error
// stream as text (decision 25).
#[cfg(target_os = "linux")]
#[test]
fn criterion_51_an_argument_error_document_that_cannot_be_written_gives_the_text_failure() {
    let home = folder();
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");
    let output = std::process::Command::new(assert_cmd::cargo::cargo_bin!("jag"))
        .env_clear()
        .env("HOME", home.path())
        .args(["--json", "--bogus"])
        .stdout(std::process::Stdio::from(full))
        .output()
        .expect("run jag");
    let run = finish(output);
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        "jag: jag could not write the results.\nCheck where the output goes.\n"
    );
}

// CRITERION-52: the version of the form.
#[test]
fn criterion_52_the_document_of_a_search_has_the_version_and_the_prompt_version() {
    let server = FakeServer::constant(0.8);
    let root = two_results();
    let document = run(&server, root.path(), &["--json"], &[], &["dir"]).document();
    assert_eq!(document["version"], 1);
    assert_eq!(document["prompt_version"], "jag-p1");
    assert_top_level(&document);
}
