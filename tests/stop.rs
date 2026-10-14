//! End-to-end tests of a stop on Ctrl-C and on a closed pipe (FEATURE-15,
//! CRITERION-64), against the fake server.
//!
//! The server holds each call for 30 seconds, so a search of 200 results
//! (7 calls, 6 at the same time) is still at its model calls when the test
//! acts. The test acts only after the server has 6 calls, and it waits up to
//! 5 seconds for the end of jag (risk 7). jag runs with `--timeout 60
//! --retry-wait 60`, so that no call times out and no retry starts.

mod common;

use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use common::fake_server::{FakeServer, Reply};
use common::{jag_std, write};
use serde_json::Value;

const QUESTION: &str = "which line says how to retry";
const SIGINT: i32 = 2;
const SIGPIPE: i32 = 13;
const STOPPED: &str = "jag: the search was stopped.\nTo get the results, run the search again.\n";

/// A server that holds each call for 30 seconds.
fn slow_server() -> FakeServer {
    FakeServer::start(|request| Reply::scores(request, |_| 0.9).after(Duration::from_secs(30)))
}

/// 200 files with one match each.
fn two_hundred_files() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("folder");
    for index in 0..200 {
        write(root.path(), &format!("dir/f{index}.txt"), b"needle\n");
    }
    root
}

/// Start `jag OPTIONS needle QUESTION dir` with its output in pipes.
fn start(server: &FakeServer, root: &Path, options: &[&str]) -> Child {
    let home = tempfile::tempdir().expect("home");
    jag_std(home.path(), &server.base_url())
        .current_dir(root)
        .args(["--timeout", "60", "--retry-wait", "60"])
        .args(options)
        .args(["needle", QUESTION, "dir"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start jag")
}

fn wait_for_calls(server: &FakeServer, count: usize) {
    let started = Instant::now();
    while server.requests().len() < count {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "the server has {} calls",
            server.requests().len()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn send_sigint(child: &Child) {
    let status = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("run kill");
    assert!(status.success());
}

/// Wait up to 5 seconds for the end of jag, and give its output.
fn finish(mut child: Child) -> Output {
    let started = Instant::now();
    while child.try_wait().expect("wait").is_none() {
        if started.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            panic!("jag did not stop in 5 seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().expect("output")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn criterion_64_ctrl_c_stops_jag_at_once_in_text_mode() {
    let server = slow_server();
    let root = two_hundred_files();
    let child = start(&server, root.path(), &[]);
    wait_for_calls(&server, 6);
    let started = Instant::now();
    send_sigint(&child);
    let output = finish(child);
    assert!(started.elapsed() < Duration::from_secs(5));
    // The end is by SIGINT, not by the code 130 (decision 13).
    assert_eq!(output.status.signal(), Some(SIGINT));
    assert_eq!(output.status.code(), None);
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), STOPPED);
    std::thread::sleep(Duration::from_secs(1));
    assert_eq!(server.requests().len(), 6);
}

#[test]
fn criterion_64_ctrl_c_gives_the_error_document_in_structured_mode() {
    let server = slow_server();
    let root = two_hundred_files();
    let child = start(&server, root.path(), &["--json"]);
    wait_for_calls(&server, 6);
    send_sigint(&child);
    let output = finish(child);
    assert_eq!(output.status.signal(), Some(SIGINT));
    assert_eq!(text(&output.stderr), "");
    let stdout = text(&output.stdout);
    assert_eq!(stdout.matches('\n').count(), 1, "{stdout}");
    let document: Value = serde_json::from_str(&stdout).expect("one document");
    assert_eq!(document["exit_code"], 130);
    assert_eq!(document["error"]["cause"], "the search was stopped.");
    assert_eq!(
        document["error"]["fix"],
        "To get the results, run the search again."
    );
    assert_eq!(document["results"], serde_json::json!([]));
    assert_eq!(document["results_found"], 200);
    assert_eq!(document["results_scored"], 200);
    assert_eq!(server.requests().len(), 6);
}

#[test]
fn criterion_64_ctrl_c_stops_the_readiness_check() {
    let server = slow_server();
    let root = tempfile::tempdir().expect("folder");
    let home = tempfile::tempdir().expect("home");
    let child = jag_std(home.path(), &server.base_url())
        .current_dir(root.path())
        .args(["--check", "--timeout", "60", "--retry-wait", "60"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start jag");
    wait_for_calls(&server, 1);
    send_sigint(&child);
    let output = finish(child);
    assert_eq!(output.status.signal(), Some(SIGINT));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(text(&output.stderr), "");
    assert_eq!(server.requests().len(), 1);
}

/// Close standard output of a search while the model scores.
fn close_the_pipe(options: &[&str]) {
    let server = slow_server();
    let root = two_hundred_files();
    let mut child = start(&server, root.path(), options);
    wait_for_calls(&server, 6);
    let started = Instant::now();
    drop(child.stdout.take());
    let output = finish(child);
    assert!(started.elapsed() < Duration::from_secs(5));
    // The end is by SIGPIPE, as grep ends, with no message (decision 15).
    assert_eq!(output.status.signal(), Some(SIGPIPE));
    assert_eq!(text(&output.stderr), "");
    std::thread::sleep(Duration::from_secs(1));
    assert_eq!(server.requests().len(), 6);
}

#[test]
fn criterion_64_a_closed_pipe_stops_jag_at_once_in_text_mode() {
    close_the_pipe(&[]);
}

#[test]
fn criterion_64_a_closed_pipe_stops_jag_at_once_in_structured_mode() {
    close_the_pipe(&["--json"]);
}
