//! End-to-end tests of FEATURE-03: the failure texts and exit codes
//! (CRITERION-13, 14, 15), the retry and the time options (CRITERION-16),
//! and a failure during a search (CRITERION-17). They use the fake server
//! only. The smallest time is 1 second, so some tests take some seconds.

mod common;

use std::path::Path;
use std::process::Output;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::fake_server::{FakeServer, Reply};
use common::{jag, jag_bare, write};
use tempfile::TempDir;

const QUESTION: &str = "which line says how to retry";
const KEYS: &str = "https://console.typesafe.ai/keys";

fn folder_with_a_match() -> TempDir {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    dir
}

/// Run `jag [extra] --ask QUESTION needle <dir>` against the server.
fn search(server: &FakeServer, dir: &Path, extra: &[&str], environment: &[(&str, &str)]) -> Output {
    search_at(&server.base_url(), dir, extra, environment)
}

fn search_at(base_url: &str, dir: &Path, extra: &[&str], environment: &[(&str, &str)]) -> Output {
    let home = tempfile::tempdir().expect("temporary folder");
    let mut command = jag(home.path(), base_url);
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        .args(extra)
        .args(["--ask", QUESTION, "needle"])
        .arg(dir)
        .output()
        .expect("run jag");
    assert_no_secret(&output);
    output
}

fn assert_no_secret(output: &Output) {
    for text in [stdout(output), stderr(output)] {
        assert!(!text.contains("SENTINEL"), "{text}");
        assert!(!text.contains("SECRET-BODY-TEXT"), "{text}");
        assert!(!text.contains("127.0.0.1"), "{text}");
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A failure has the exit code, no output, and exactly the cause and the
/// fix on the error stream (CRITERION-13, 14, 15).
fn assert_failure(output: &Output, code: i32, cause: &str, fix: &str) {
    assert_eq!(stderr(output), format!("jag: {cause}\n{fix}\n"));
    assert_eq!(output.status.code(), Some(code));
    assert!(stdout(output).is_empty());
}

/// A server that gives the same status and a body with a secret to each call.
fn status_server(status: u16) -> FakeServer {
    FakeServer::start(move |_| Reply::status(status).body("SECRET-BODY-TEXT"))
}

// CRITERION-13, 14, 15: one case for each row of the failure table.
#[test]
fn no_key_gives_exit_3_and_the_setup_fix() {
    let dir = folder_with_a_match();
    let home = tempfile::tempdir().expect("temporary folder");
    let output = jag_bare(home.path())
        .args(["--ask", QUESTION, "needle"])
        .arg(dir.path())
        .output()
        .expect("run jag");
    assert_failure(
        &output,
        3,
        "no TypeSafe API key found.",
        &format!("Get a key at {KEYS}, then run `jag --setup`,\nor set JAG_TYPESAFE_API_KEY."),
    );
}

#[test]
fn http_401_and_403_give_exit_4() {
    for status in [401, 403] {
        let server = status_server(status);
        let dir = folder_with_a_match();
        let output = search(&server, dir.path(), &[], &[]);
        assert_failure(
            &output,
            4,
            &format!("the gateway did not accept the key (HTTP {status})."),
            "Check that the key is a key of the gateway, then run jag --setup.",
        );
        assert_eq!(server.requests().len(), 1);
    }
}

#[test]
fn http_402_gives_exit_5() {
    let server = status_server(402);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        5,
        "the gateway account has no credit (HTTP 402).",
        "Add credit at the gateway, then try again.",
    );
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn a_timeout_gives_exit_6_after_one_retry() {
    let server = FakeServer::start(|_| Reply::status(200).after(Duration::from_secs(3)));
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &["--timeout", "1"], &[]);
    assert_failure(
        &output,
        6,
        "the gateway did not answer in time.",
        "Try again later, or give each call more time with --timeout.",
    );
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn no_connection_gives_exit_6() {
    // A port that nothing listens on.
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.local_addr().expect("address").port()
    };
    let dir = folder_with_a_match();
    let output = search_at(&format!("http://127.0.0.1:{port}/v1"), dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "jag cannot connect to the gateway.",
        "Check the network connection and JAG_BASE_URL, then try again.",
    );
}

#[test]
fn http_429_gives_exit_6_after_one_retry() {
    let server = status_server(429);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway refused more calls (HTTP 429).",
        "Wait one minute, then try again.",
    );
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn http_5xx_gives_exit_6_after_one_retry() {
    let server = status_server(503);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway failed (HTTP 503).",
        "Try again later.",
    );
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn a_redirect_gives_exit_6() {
    let server = status_server(302);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway sent a redirect.",
        "Check JAG_BASE_URL.",
    );
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn another_status_gives_exit_6() {
    let server = status_server(422);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway gave HTTP 422.",
        "Check JAG_BASE_URL, and set the model name of the gateway with --model, JAG_MODEL, or config.model.",
    );
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn a_body_that_jag_cannot_read_gives_exit_6() {
    let server = FakeServer::start(|_| Reply::status(200).body("SECRET-BODY-TEXT"));
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway gave a response that jag cannot read.",
        "Check JAG_BASE_URL.",
    );
}

// CRITERION-14: 0, 1, and 2 keep their grep meaning.
#[test]
fn a_search_with_results_exits_0_no_match_exits_1_and_a_bad_term_exits_2() {
    let server = FakeServer::constant(0.8);
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("a.txt:1:needle"));

    let home = tempfile::tempdir().expect("temporary folder");
    let no_match = jag(home.path(), &server.base_url())
        .args(["--ask", QUESTION, "absent"])
        .arg(dir.path())
        .output()
        .expect("run jag");
    assert_eq!(no_match.status.code(), Some(1));

    let bad_term = jag(home.path(), &server.base_url())
        .args(["--ask", QUESTION, "("])
        .arg(dir.path())
        .output()
        .expect("run jag");
    assert_eq!(bad_term.status.code(), Some(2));
}

/// A server that gives 500, then good scores. It keeps the time of each call.
fn failing_once_server() -> (FakeServer, Arc<Mutex<Vec<Instant>>>) {
    let times = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&times);
    let server = FakeServer::start(move |request| {
        let mut times = seen.lock().expect("lock");
        times.push(Instant::now());
        if times.len() == 1 {
            Reply::status(500)
        } else {
            Reply::scores(request, |_| 0.8)
        }
    });
    (server, times)
}

/// The time between the first and the second call.
fn gap(times: &Arc<Mutex<Vec<Instant>>>) -> Duration {
    let times = times.lock().expect("lock");
    assert_eq!(times.len(), 2, "two calls");
    times[1] - times[0]
}

// CRITERION-16: one retry, and the user can change the wait.
#[test]
fn a_500_then_a_200_gives_exit_0_after_two_calls() {
    let (server, times) = failing_once_server();
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(gap(&times) >= Duration::from_secs(1));
}

#[test]
fn the_retry_wait_option_sets_the_wait() {
    let (server, times) = failing_once_server();
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &["--retry-wait", "2"], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(gap(&times) >= Duration::from_secs(2));
}

#[test]
fn the_retry_wait_variable_sets_the_wait() {
    let (server, times) = failing_once_server();
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &[], &[("JAG_RETRY_WAIT", "2")]);
    assert_eq!(output.status.code(), Some(0));
    assert!(gap(&times) >= Duration::from_secs(2));
}

#[test]
fn the_retry_wait_option_wins_over_the_variable() {
    let (server, times) = failing_once_server();
    let dir = folder_with_a_match();
    let output = search(
        &server,
        dir.path(),
        &["--retry-wait", "1"],
        &[("JAG_RETRY_WAIT", "3")],
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(gap(&times) < Duration::from_secs(3));
}

/// A server that waits 3 seconds to answer each call.
fn slow_server() -> FakeServer {
    FakeServer::start(|request| Reply::scores(request, |_| 0.8).after(Duration::from_secs(3)))
}

fn assert_timed_out_after_two_calls(server: &FakeServer, output: &Output, started: Instant) {
    assert_eq!(output.status.code(), Some(6));
    assert!(stdout(output).is_empty());
    assert_eq!(server.requests().len(), 2);
    // 1 s, the 1 s wait, and 1 s, with room for a slow machine.
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn the_timeout_option_sets_the_time_of_one_call() {
    let server = slow_server();
    let dir = folder_with_a_match();
    let started = Instant::now();
    let output = search(&server, dir.path(), &["--timeout", "1"], &[]);
    assert_timed_out_after_two_calls(&server, &output, started);
}

#[test]
fn the_timeout_variable_sets_the_time_of_one_call() {
    let server = slow_server();
    let dir = folder_with_a_match();
    let started = Instant::now();
    let output = search(&server, dir.path(), &[], &[("JAG_TIMEOUT", "1")]);
    assert_timed_out_after_two_calls(&server, &output, started);
}

#[test]
fn the_timeout_option_wins_over_the_variable() {
    let server = slow_server();
    let dir = folder_with_a_match();
    let started = Instant::now();
    let output = search(
        &server,
        dir.path(),
        &["--timeout", "1"],
        &[("JAG_TIMEOUT", "30")],
    );
    assert_timed_out_after_two_calls(&server, &output, started);
}

#[test]
fn a_longer_timeout_waits_for_a_slow_reply() {
    // The default timeout of 4 s is too short for this reply.
    let server = FakeServer::start(|request| {
        Reply::scores(request, |_| 0.8).after(Duration::from_millis(4500))
    });
    let dir = folder_with_a_match();
    let output = search(&server, dir.path(), &["--timeout", "7"], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(server.requests().len(), 1);
}

// The deadline is `2 x timeout + retry wait` from the first call.
#[test]
fn the_search_fails_within_the_deadline_of_the_times() {
    let server = FakeServer::start(|_| Reply::status(200).after(Duration::from_secs(10)));
    let dir = folder_with_a_match();
    let started = Instant::now();
    let output = search(
        &server,
        dir.path(),
        &["--timeout", "1", "--retry-wait", "2"],
        &[],
    );
    assert_eq!(output.status.code(), Some(6));
    assert_eq!(server.requests().len(), 2);
    // The formula gives 4 s. 1 s is room for a slow machine.
    assert!(started.elapsed() < Duration::from_secs(5));
}

// CRITERION-17: a failure during a search fails the full search.
fn many_files() -> TempDir {
    let dir = tempfile::tempdir().expect("temporary folder");
    for number in 0..40 {
        write(dir.path(), &format!("f{number:02}.txt"), b"needle\n");
    }
    dir
}

/// The first request gets scores. Each later request gets `status`.
fn first_call_only_server(status: u16) -> FakeServer {
    let seen = Mutex::new(0);
    FakeServer::start(move |request| {
        let mut seen = seen.lock().expect("lock");
        *seen += 1;
        if *seen == 1 {
            Reply::scores(request, |_| 0.8)
        } else {
            Reply::status(status)
        }
    })
}

#[test]
fn a_402_during_the_search_fails_the_full_search() {
    let server = first_call_only_server(402);
    let dir = many_files();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        5,
        "the gateway account has no credit (HTTP 402).",
        "Add credit at the gateway, then try again.",
    );
    assert!(server.requests().len() >= 2);
}

#[test]
fn a_500_during_the_search_fails_the_full_search() {
    let server = first_call_only_server(500);
    let dir = many_files();
    let output = search(&server, dir.path(), &[], &[]);
    assert_failure(
        &output,
        6,
        "the gateway failed (HTTP 500).",
        "Try again later.",
    );
}
