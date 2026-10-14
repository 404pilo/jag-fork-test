//! End-to-end tests of `jag --ask "QUESTION" TERM PATH...` against the fake server.
//!
//! Each test sets HOME to a temporary folder, uses the made-up key of
//! `common::KEY`, and calls only the fake server on 127.0.0.1.

mod common;

use std::path::Path;
use std::sync::Mutex;

use common::fake_server::{FakeServer, Reply};
use common::{KEY, jag, jag_bare, write};
use jag::output::NO_GOOD_ANSWER;
use predicates::prelude::*;
use tempfile::TempDir;

const QUESTION: &str = "which line says how to retry";

fn folder() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

/// Run a search and give the output. The key never shows in the output or
/// in a body that the server got.
fn search(
    server: &FakeServer,
    term: &str,
    root: &Path,
    environment: &[(&str, &str)],
) -> std::process::Output {
    let home = folder();
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        // These tests are not about the size limit. The long temporary paths of
        // macOS can make many results larger than the default limit.
        .args(["--no-max-chars", "--ask", QUESTION, term])
        .arg(root)
        .output()
        .expect("run jag");
    assert_no_key(&output, server);
    output
}

fn assert_no_key(output: &std::process::Output, server: &FakeServer) {
    assert!(!String::from_utf8_lossy(&output.stdout).contains(KEY));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(KEY));
    for request in server.requests() {
        assert!(!request.body_text().contains("SENTINEL"));
    }
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// CRITERION-18: the command takes the term, the question, and the paths.
#[test]
fn criterion_18_the_command_takes_a_term_a_question_and_a_path() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("a.txt:1:needle"));
    assert_eq!(server.requests().len(), 1);
    assert_eq!(server.requests()[0].json()["state"]["question"], QUESTION);
}

// CRITERION-19: the best answer comes first.
#[test]
fn criterion_19_the_best_answer_comes_first() {
    let server = FakeServer::start(|request| {
        Reply::scores(request, |text| {
            if text.contains("top") {
                0.9
            } else if text.contains("mid") {
                0.3
            } else {
                0.1
            }
        })
    });
    let dir = folder();
    write(dir.path(), "a.txt", b"needle low\n");
    write(dir.path(), "b.txt", b"needle top\n");
    write(dir.path(), "c.txt", b"needle mid\n");
    let output = search(&server, "needle", dir.path(), &[]);
    let text = stdout(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].ends_with("b.txt:1:needle top"), "{text}");
    assert_eq!(lines[1], "--");
    assert!(lines[2].ends_with("c.txt:1:needle mid"), "{text}");
    assert_eq!(lines[3], "--");
    assert!(lines[4].ends_with("a.txt:1:needle low"), "{text}");
    // The top score 0.9 is a good answer: no label and exit 0.
    assert_eq!(output.status.code(), Some(0));
}

// CRITERION-20: the term is a regular expression, as in grep.
#[test]
fn criterion_20_the_term_is_a_case_sensitive_regular_expression() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\nneeedle\nNeedle\nnodle\n");
    let output = search(&server, "ne+dle", dir.path(), &[]);
    let text = stdout(&output);
    assert!(text.contains(":1:needle"), "{text}");
    assert!(text.contains(":2:neeedle"), "{text}");
    // The lines around a match show with a `-`, so test the match marks.
    assert!(!text.contains(":3:Needle"), "{text}");
    assert!(!text.contains(":4:nodle"), "{text}");
}

// CRITERION-21: a result holds the lines around the match.
#[test]
fn criterion_21_a_result_holds_the_lines_around_the_match() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    let text: String = (1..=40)
        .map(|number| {
            if number == 10 || number == 30 {
                format!("needle {number}\n")
            } else {
                format!("line {number}\n")
            }
        })
        .collect();
    write(dir.path(), "a.txt", text.as_bytes());
    let output = search(&server, "needle", dir.path(), &[]);
    let out = stdout(&output);
    assert!(out.contains("a.txt-9-line 9\n"), "{out}");
    assert!(out.contains("a.txt:10:needle 10\n"), "{out}");
    assert!(out.contains("a.txt-13-line 13\n"), "{out}");
    assert!(!out.contains("a.txt-14-"), "{out}");
    assert!(out.contains("\n--\n"), "{out}");
    assert!(out.contains("a.txt:30:needle 30\n"), "{out}");
}

// CRITERION-29 and CRITERION-30: the label and exit 1.
#[test]
fn criterion_29_weak_results_have_a_clear_label_and_criterion_30_exit_1() {
    let server = FakeServer::constant(0.2);
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let out = stdout(&output);
    assert_eq!(out.lines().next(), Some(NO_GOOD_ANSWER));
    assert!(out.contains("a.txt:1:needle"), "{out}");
}

#[test]
fn criterion_30_the_threshold_is_0_4_and_a_score_of_0_4_is_a_good_answer() {
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");

    let server = FakeServer::constant(0.4);
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!stdout(&output).contains(NO_GOOD_ANSWER));

    let server = FakeServer::constant(0.39);
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).starts_with(NO_GOOD_ANSWER));
}

// CRITERION-71: a term with no matches makes no model call.
#[test]
fn criterion_71_no_match_makes_no_call_and_exits_1() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), "a.txt", b"nothing here\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).is_empty());
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_71_with_no_key_and_no_match_the_exit_is_3() {
    let dir = folder();
    write(dir.path(), "a.txt", b"nothing here\n");
    let home = folder();
    jag_bare(home.path())
        .args(["--ask", QUESTION, "needle"])
        .arg(dir.path())
        .assert()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("no TypeSafe API key"));
}

// CRITERION-85: a ranked search says how many files it skipped.
#[test]
fn criterion_85_the_notice_says_how_many_files_jag_skipped() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), ".hidden.txt", b"needle\n");
    write(dir.path(), "bin.dat", b"needle\n\0");
    write(dir.path(), "ok.txt", b"needle\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stderr(&output)
            .contains("jag: skipped 2 files and folders (1 binary, 1 hidden, 0 ignored)"),
        "{}",
        stderr(&output)
    );
    // The notice is not on standard output.
    assert!(!stdout(&output).contains("skipped"));
}

#[test]
fn criterion_85_the_notice_shows_when_no_file_matches() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), ".hidden.txt", b"needle\n");
    write(dir.path(), "ok.txt", b"nothing\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output)
            .contains("jag: skipped 1 files and folders (0 binary, 1 hidden, 0 ignored)")
    );
    assert!(server.requests().is_empty());
}

#[test]
fn a_search_in_several_paths_scores_the_results_of_each() {
    let server = FakeServer::constant(0.8);
    let first = folder();
    let second = folder();
    write(first.path(), "a.txt", b"needle one\n");
    write(second.path(), "b.txt", b"needle two\n");
    let home = folder();
    let output = jag(home.path(), &server.base_url())
        .args(["--ask", QUESTION, "needle"])
        .arg(first.path())
        .arg(second.path())
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(0));
    let out = stdout(&output);
    assert!(
        out.contains("needle one") && out.contains("needle two"),
        "{out}"
    );
}

#[test]
fn the_default_limit_scores_200_results() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    for number in 0..250 {
        write(dir.path(), &format!("f{number:03}.txt"), b"needle\n");
    }
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(7));
    assert!(
        stderr(&output).contains("jag: cost limit reached: scored 200 of 250 results, skipped 50.")
    );
    let sent: usize = server
        .requests()
        .iter()
        .map(|request| request.snippet_texts().len())
        .sum();
    assert_eq!(sent, 200);
    // One match line in each result.
    let matches = stdout(&output)
        .lines()
        .filter(|line| line.ends_with(":1:needle"))
        .count();
    assert_eq!(matches, 200);
}

#[test]
fn a_long_line_is_cut_and_marked() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    let long = format!("needle{}", "x".repeat(994));
    write(dir.path(), "a.txt", format!("{long}\n").as_bytes());
    let output = search_from(&server, dir.path(), &["a.txt"]);
    assert_eq!(output.status.code(), Some(0));
    let kept = format!("needle{}", "x".repeat(294));
    let request_text = server.requests()[0].snippet_texts()[0].clone();
    assert_eq!(request_text.chars().count(), 300);
    assert_eq!(request_text, kept);
    assert_eq!(stdout(&output), format!("a.txt:1:{kept} [+700 chars]\n"));
}

/// Run a search with the paths as typed, from the folder `cwd`.
fn search_from(server: &FakeServer, cwd: &Path, paths: &[&str]) -> std::process::Output {
    let home = folder();
    let output = jag(home.path(), &server.base_url())
        .current_dir(cwd)
        .args(["--ask", QUESTION, "needle"])
        .args(paths)
        .output()
        .expect("run jag");
    assert_no_key(&output, server);
    output
}

// CRITERION-31: each line shows the path, the line number, and the text; a
// matching line has `:` and another line has `-`.
#[test]
fn criterion_31_each_line_has_the_path_the_number_and_a_separator() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    let lines: Vec<String> = (1..=8)
        .map(|number| {
            if number == 5 {
                "needle".to_string()
            } else {
                format!("line {number}")
            }
        })
        .collect();
    write(
        dir.path(),
        "notes/a.txt",
        format!("{}\n", lines.join("\n")).as_bytes(),
    );
    let output = search_from(&server, dir.path(), &["notes"]);
    assert_eq!(output.status.code(), Some(0));
    let expected: String = (2..=8)
        .map(|number| {
            let separator = if number == 5 { ':' } else { '-' };
            format!(
                "notes/a.txt{separator}{number}{separator}{}\n",
                lines[number - 1]
            )
        })
        .collect();
    assert_eq!(stdout(&output), expected);
}

// CRITERION-32: the path is as the user typed it, in each form.
#[test]
fn criterion_32_the_path_has_the_form_that_the_user_typed() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), "dir/sub/a.txt", b"needle\n");
    let absolute = dir.path().join("dir");
    let absolute = absolute.to_string_lossy().into_owned();
    let cases = [
        ("dir", "dir/sub/a.txt:1:needle\n".to_string()),
        ("dir/", "dir/sub/a.txt:1:needle\n".to_string()),
        ("./dir", "./dir/sub/a.txt:1:needle\n".to_string()),
        (
            absolute.as_str(),
            format!("{absolute}/sub/a.txt:1:needle\n"),
        ),
        ("dir/sub/a.txt", "dir/sub/a.txt:1:needle\n".to_string()),
    ];
    for (typed, expected) in cases {
        let output = search_from(&server, dir.path(), &[typed]);
        assert_eq!(output.status.code(), Some(0), "{typed}");
        assert_eq!(stdout(&output), expected, "{typed}");
    }
}

// CRITERION-33: the result runs from the first line to the last line of its
// window, so two near matches give one result.
#[test]
fn criterion_33_a_result_runs_from_its_first_line_to_its_last_line() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    let text: String = (1..=20)
        .map(|number| {
            if number == 10 || number == 14 {
                "needle\n".to_string()
            } else {
                format!("line {number}\n")
            }
        })
        .collect();
    write(dir.path(), "a.txt", text.as_bytes());
    let output = search_from(&server, dir.path(), &["a.txt"]);
    let out = stdout(&output);
    let rows: Vec<&str> = out.lines().collect();
    assert_eq!(rows.len(), 11);
    assert!(rows[0].starts_with("a.txt-7-"));
    assert!(rows[10].starts_with("a.txt-17-"));
    assert!(rows[3].starts_with("a.txt:10:"));
    assert!(rows[7].starts_with("a.txt:14:"));
    assert!(!out.contains("--\n"));
}

#[test]
fn the_request_has_the_model_from_the_variable() {
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    search(
        &server,
        "needle",
        dir.path(),
        &[("JAG_MODEL", "jev-latest")],
    );
    let body = server.requests()[0].json();
    assert_eq!(body["model"], "jev-latest");
    assert_eq!(body["questions"]["s1"]["type"], "noul");
    assert_eq!(
        body["questions"]["s1"]["instructions"],
        "Does snippet 1 answer the question?"
    );
}

#[test]
fn a_file_that_jag_cannot_read_gives_a_warning_and_the_others_are_scored() {
    use std::os::unix::fs::PermissionsExt;
    let server = FakeServer::constant(0.8);
    let dir = folder();
    write(dir.path(), ".hidden.txt", b"needle\n");
    write(dir.path(), "ok.txt", b"needle\n");
    write(dir.path(), "locked.txt", b"needle\n");
    let locked = dir.path().join("locked.txt");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("lock");
    // A user who reads any file (root) cannot run this test.
    let can_read = std::fs::read(&locked).is_ok();
    let output = search(&server, "needle", dir.path(), &[]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).expect("unlock");
    if can_read {
        return;
    }
    assert_eq!(output.status.code(), Some(0));
    let err = stderr(&output);
    let notice = err.find("jag: skipped").expect("the notice");
    let warning = err.find("jag: cannot read").expect("the warning");
    assert!(notice < warning, "{err}");
    assert!(stdout(&output).contains("ok.txt:1:needle"));
    assert!(!stdout(&output).contains("locked.txt"));
}

// Failures: a model failure fails the full search with no partial result.
#[test]
fn a_model_failure_gives_its_exit_code_and_no_output() {
    // (status, exit code, calls)
    let cases = [
        (401, 4, 1),
        (403, 4, 1),
        (402, 5, 1),
        (422, 6, 1),
        (302, 6, 1),
    ];
    for (status, code, calls) in cases {
        let server = FakeServer::start(move |_| Reply::status(status).body("SECRET-BODY-TEXT"));
        let dir = folder();
        write(dir.path(), "a.txt", b"needle\n");
        let output = search(&server, "needle", dir.path(), &[]);
        assert_eq!(output.status.code(), Some(code), "status {status}");
        assert!(stdout(&output).is_empty(), "status {status}");
        assert!(!stderr(&output).contains("SECRET-BODY-TEXT"));
        assert!(!stderr(&output).contains("127.0.0.1"));
        assert_eq!(server.requests().len(), calls, "status {status}");
    }
}

#[test]
fn the_exit_4_text_says_how_to_fix_the_key() {
    let server = FakeServer::start(|_| Reply::status(401));
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    let output = search(&server, "needle", dir.path(), &[]);
    let err = stderr(&output);
    assert!(err.contains("did not accept the key (HTTP 401)"), "{err}");
    assert!(err.contains("https://console.typesafe.ai/keys"), "{err}");
    assert!(err.contains("jag --setup"), "{err}");
}

#[test]
fn a_failure_that_the_retry_cannot_fix_gives_exit_6_after_two_calls() {
    for status in [500, 429] {
        let server = FakeServer::start(move |_| Reply::status(status));
        let dir = folder();
        write(dir.path(), "a.txt", b"needle\n");
        let output = search(&server, "needle", dir.path(), &[]);
        assert_eq!(output.status.code(), Some(6), "status {status}");
        assert!(stdout(&output).is_empty());
        assert_eq!(server.requests().len(), 2, "status {status}");
    }
}

#[test]
fn a_failure_then_a_success_gives_the_results() {
    let seen = Mutex::new(0);
    let server = FakeServer::start(move |request| {
        let mut seen = seen.lock().expect("lock");
        *seen += 1;
        if *seen == 1 {
            Reply::status(500)
        } else {
            Reply::scores(request, |_| 0.8)
        }
    });
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    let output = search(&server, "needle", dir.path(), &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("a.txt:1:needle"));
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn a_body_that_jag_cannot_read_gives_exit_6() {
    let bodies = [
        "SECRET-BODY-TEXT",
        r#"{"answers":{}}"#,
        r#"{"answers":{"s1":{"type":"noul","noul":1.5}}}"#,
    ];
    for body in bodies {
        let server = FakeServer::start(move |_| Reply::status(200).body(body));
        let dir = folder();
        write(dir.path(), "a.txt", b"needle\n");
        let output = search(&server, "needle", dir.path(), &[]);
        assert_eq!(output.status.code(), Some(6), "body {body}");
        assert!(stdout(&output).is_empty());
        assert!(!stderr(&output).contains("SECRET-BODY-TEXT"));
        assert!(!stderr(&output).contains("noul"));
    }
}

// TLS: jag sets up TLS before the handshake. Without the `platform-verifier`
// feature of `ureq`, this test panics (the pull request says so).
#[test]
fn an_https_address_that_closes_the_connection_gives_exit_6_and_no_panic() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    // Accept and close each connection. The thread ends with the test.
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            drop(stream);
        }
    });
    let dir = folder();
    write(dir.path(), "a.txt", b"needle\n");
    let home = folder();
    let output = jag(home.path(), &format!("https://127.0.0.1:{port}/v1"))
        .args(["--ask", QUESTION, "needle"])
        .arg(dir.path())
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    assert!(!stderr(&output).contains("panicked"), "{}", stderr(&output));
    assert!(stdout(&output).is_empty());
}
