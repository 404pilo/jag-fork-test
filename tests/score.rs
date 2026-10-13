//! Tests of `score`: the calls, the retry, the stop flag, the deadline, and
//! the map from a status to a failure. They call the fake server only.

mod common;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use common::fake_server::{FakeServer, Reply};
use jag::config::ApiKey;
use jag::failure::Failure;
use jag::scorer::{ScoreError, ScoreSettings, score};
use jag::search::snippet::{Snippet, SnippetLine};

const KEY: &str = "SENTINEL-SENTINEL-SENTINEL";

fn snippets(count: usize) -> Vec<Snippet> {
    (0..count)
        .map(|index| Snippet {
            path: PathBuf::from(format!("f{index}.txt")),
            lines: vec![SnippetLine {
                number: 1,
                text: format!("text-{index}").into_bytes(),
                is_match: true,
            }],
        })
        .collect()
}

/// Settings with short times, so that the tests are fast.
fn settings(server: &FakeServer) -> ScoreSettings {
    ScoreSettings {
        base_url: server.base_url(),
        model: "jev-1.13.0".to_owned(),
        timeout: Duration::from_secs(2),
        retry_wait: Duration::from_millis(20),
        deadline: Duration::from_secs(5),
    }
}

fn run(
    server: &FakeServer,
    snippets: &[Snippet],
    settings: &ScoreSettings,
) -> Result<Vec<f64>, ScoreError> {
    let key = ApiKey::parse(KEY).expect("a test key");
    let _ = server;
    score(snippets, "which one answers?", &key, settings)
}

fn exit_code(error: ScoreError) -> u8 {
    Failure::from(error).exit_code
}

#[test]
fn the_scores_come_back_in_the_order_of_the_snippets() {
    let server = FakeServer::start(|request| {
        Reply::scores(request, |text| {
            let index: f64 = text.trim_start_matches("text-").parse().unwrap_or(0.0);
            index / 1000.0
        })
    });
    let scores = run(&server, &snippets(70), &settings(&server)).expect("scores");
    assert_eq!(scores.len(), 70);
    for (index, value) in scores.iter().enumerate() {
        assert_eq!(*value, index as f64 / 1000.0);
    }
    // 70 short snippets: calls of 32, 32, and 6.
    let mut sizes: Vec<usize> = server
        .requests()
        .iter()
        .map(|request| request.snippet_texts().len())
        .collect();
    sizes.sort_unstable();
    assert_eq!(sizes, [6, 32, 32]);
}

#[test]
fn no_snippet_makes_no_call() {
    let server = FakeServer::constant(0.5);
    let scores = run(&server, &[], &settings(&server)).expect("scores");
    assert!(scores.is_empty());
    assert!(server.requests().is_empty());
}

#[test]
fn the_key_goes_only_in_the_authorization_header() {
    let server = FakeServer::constant(0.5);
    run(&server, &snippets(3), &settings(&server)).expect("scores");
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/systemone");
    assert_eq!(
        request.header("authorization"),
        Some(&*format!("Bearer {KEY}"))
    );
    assert_eq!(request.header("content-type"), Some("application/json"));
    assert_eq!(request.header("accept"), Some("application/json"));
    assert!(
        request
            .header("user-agent")
            .is_some_and(|agent| agent.starts_with("jag/"))
    );
    assert!(!request.body_text().contains("SENTINEL"));
    for (name, value) in &request.headers {
        if name != "authorization" {
            assert!(!value.contains("SENTINEL"), "key in header {name}");
        }
    }
}

#[test]
fn the_request_has_the_model_the_question_and_one_question_for_each_snippet() {
    let server = FakeServer::constant(0.5);
    run(&server, &snippets(2), &settings(&server)).expect("scores");
    let body = server.requests()[0].json();
    assert_eq!(body["model"], "jev-1.13.0");
    assert_eq!(body["state"]["question"], "which one answers?");
    assert_eq!(body["questions"]["s1"]["type"], "noul");
    assert_eq!(
        body["questions"]["s2"]["instructions"],
        "Does snippet 2 answer the question?"
    );
}

#[test]
fn a_status_gives_its_failure_and_its_retry() {
    // (status, error, calls)
    let cases = [
        (401, ScoreError::KeyNotAccepted { status: 401 }, 1),
        (403, ScoreError::KeyNotAccepted { status: 403 }, 1),
        (402, ScoreError::NoCredit, 1),
        (500, ScoreError::ServerFailed { status: 500 }, 2),
        (503, ScoreError::ServerFailed { status: 503 }, 2),
        (429, ScoreError::RateLimited, 2),
        (422, ScoreError::UnexpectedStatus { status: 422 }, 1),
        (408, ScoreError::UnexpectedStatus { status: 408 }, 1),
        (302, ScoreError::Redirect, 1),
    ];
    for (status, error, calls) in cases {
        let server = FakeServer::start(move |_| Reply::status(status));
        let result = run(&server, &snippets(1), &settings(&server));
        assert_eq!(result, Err(error), "status {status}");
        assert_eq!(server.requests().len(), calls, "calls for status {status}");
    }
}

#[test]
fn the_exit_codes_are_4_5_and_6() {
    assert_eq!(exit_code(ScoreError::KeyNotAccepted { status: 401 }), 4);
    assert_eq!(exit_code(ScoreError::NoCredit), 5);
    assert_eq!(exit_code(ScoreError::TimedOut), 6);
}

#[test]
fn a_failure_then_a_success_gives_the_scores() {
    let seen = Mutex::new(0);
    let server = FakeServer::start(move |request| {
        let mut seen = seen.lock().expect("lock");
        *seen += 1;
        if *seen == 1 {
            Reply::status(500)
        } else {
            Reply::scores(request, |_| 0.7)
        }
    });
    let scores = run(&server, &snippets(1), &settings(&server)).expect("scores");
    assert_eq!(scores, [0.7]);
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn a_body_that_jag_cannot_read_gives_exit_6_with_no_retry() {
    let bodies = [
        "not json".to_owned(),
        r#"{"answers":{}}"#.to_owned(),
        r#"{"answers":{"s1":{"type":"noul","noul":1.5}}}"#.to_owned(),
    ];
    for body in bodies {
        let reply_body = body.clone();
        let server = FakeServer::start(move |_| Reply::status(200).body(&reply_body));
        let result = run(&server, &snippets(1), &settings(&server));
        assert_eq!(result, Err(ScoreError::BadResponse), "body {body}");
        assert_eq!(server.requests().len(), 1);
    }
}

#[test]
fn a_failure_text_never_holds_the_body_or_the_address() {
    let server = FakeServer::start(|_| Reply::status(500).body("SECRET-BODY-TEXT"));
    let error = run(&server, &snippets(1), &settings(&server)).expect_err("a failure");
    let text = Failure::from(error).render();
    assert!(!text.contains("SECRET-BODY-TEXT"));
    assert!(!text.contains("127.0.0.1"));
    assert!(!text.contains("SENTINEL"));
}

#[test]
fn a_slow_server_gives_a_timeout_after_two_calls() {
    let server =
        FakeServer::start(|request| Reply::scores(request, |_| 0.5).after(Duration::from_secs(1)));
    let mut settings = settings(&server);
    settings.timeout = Duration::from_millis(200);
    settings.retry_wait = Duration::from_millis(50);
    settings.deadline = 2 * settings.timeout + settings.retry_wait;
    let started = Instant::now();
    let error = run(&server, &snippets(1), &settings).expect_err("a timeout");
    assert_eq!(error, ScoreError::TimedOut);
    assert_eq!(exit_code(error), 6);
    assert_eq!(server.requests().len(), 2);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn no_connection_gives_exit_6() {
    // A port that nothing listens on: bind, read the port, and drop.
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.local_addr().expect("address").port()
    };
    let key = ApiKey::parse(KEY).expect("a test key");
    let settings = ScoreSettings {
        base_url: format!("http://127.0.0.1:{port}/v1"),
        model: "m".to_owned(),
        timeout: Duration::from_secs(1),
        retry_wait: Duration::from_millis(20),
        deadline: Duration::from_secs(5),
    };
    let error = score(&snippets(1), "q", &key, &settings).expect_err("no connection");
    assert_eq!(error, ScoreError::CannotConnect);
}

#[test]
fn after_the_first_failure_no_call_and_no_retry_starts() {
    // 320 short snippets: 10 calls. The server finds the first call by its
    // content, not by the order of the arrival.
    let mut input = snippets(320);
    input[0].lines[0].text = b"FIRST-CALL-MARK".to_vec();
    let server = FakeServer::start(|request| {
        let first = request
            .snippet_texts()
            .first()
            .is_some_and(|text| text == "FIRST-CALL-MARK");
        if first {
            Reply::status(401)
        } else {
            Reply::status(500).after(Duration::from_millis(100))
        }
    });
    let mut settings = settings(&server);
    settings.deadline = Duration::from_secs(5);
    let error = run(&server, &input, &settings).expect_err("a failure");
    assert_eq!(exit_code(error), 4);
    let requests = server.requests();
    assert!(requests.len() <= 6, "{} calls", requests.len());
    // No retry: no first snippet text comes two times.
    let firsts: Vec<String> = requests
        .iter()
        .filter_map(|request| request.snippet_texts().first().cloned())
        .collect();
    let unique: HashSet<&String> = firsts.iter().collect();
    assert_eq!(unique.len(), firsts.len());
}

#[test]
fn no_more_than_six_calls_run_at_the_same_time() {
    // 640 short snippets: 20 calls.
    let server = FakeServer::start(|request| {
        Reply::scores(request, |_| 0.5).after(Duration::from_millis(100))
    });
    let mut settings = settings(&server);
    settings.deadline = Duration::from_secs(5);
    let scores = run(&server, &snippets(640), &settings).expect("scores");
    assert_eq!(scores.len(), 640);
    assert_eq!(server.requests().len(), 20);
    let most = server.most_at_once();
    assert!(most <= 6, "{most} calls at the same time");
    assert!(most >= 2, "the calls did not run in parallel");
}

/// 8 calls. Each reply waits 150 ms. The first reply to calls 7 and 8 is 500,
/// and the second reply is 200. The retries start at about 350 ms.
fn deadline_server() -> FakeServer {
    let seen = Mutex::new(HashSet::new());
    FakeServer::start(move |request| {
        let texts = request.snippet_texts();
        let late = texts
            .first()
            .is_some_and(|text| text == "text-192" || text == "text-224");
        let first_time = seen
            .lock()
            .expect("lock")
            .insert(texts.first().cloned().unwrap_or_default());
        let reply = if late && first_time {
            Reply::status(500)
        } else {
            Reply::scores(request, |_| 0.5)
        };
        reply.after(Duration::from_millis(150))
    })
}

fn deadline_settings(server: &FakeServer, deadline: Duration) -> ScoreSettings {
    ScoreSettings {
        base_url: server.base_url(),
        model: "m".to_owned(),
        // A long timeout, so that only the deadline can stop a retry.
        timeout: Duration::from_secs(1),
        retry_wait: Duration::from_millis(50),
        deadline,
    }
}

#[test]
fn the_deadline_stops_a_retry_that_has_no_time_left() {
    let server = deadline_server();
    let settings = deadline_settings(&server, Duration::from_millis(450));
    let started = Instant::now();
    let error = run(&server, &snippets(256), &settings).expect_err("a timeout");
    assert_eq!(exit_code(error), 6);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn with_no_deadline_the_same_retries_succeed() {
    let server = deadline_server();
    let settings = deadline_settings(&server, Duration::from_secs(5));
    let scores = run(&server, &snippets(256), &settings).expect("scores");
    assert_eq!(scores.len(), 256);
}

#[test]
fn a_skipped_retry_does_not_hide_the_first_failure() {
    // 2 calls. Call 1 gets 503 and would retry after 400 ms. Call 2 gets 401
    // at once and sets the stop flag, so the retry of call 1 is skipped. The
    // exit code is 4 on each run.
    for _ in 0..5 {
        let server = FakeServer::start(|request| {
            let first = request.snippet_texts().into_iter().next();
            if first.as_deref() == Some("text-0") {
                Reply::status(503)
            } else {
                Reply::status(401)
            }
        });
        let mut settings = settings(&server);
        settings.retry_wait = Duration::from_millis(400);
        let error = run(&server, &snippets(64), &settings).expect_err("a failure");
        assert_eq!(exit_code(error), 4);
        assert_eq!(server.requests().len(), 2);
    }
}
