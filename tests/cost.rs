//! End-to-end tests of FEATURE-20: the cost line on the error stream and the
//! `cost` field of the document (CRITERION-88 to 93, and 97), against the
//! fake server.
//!
//! 33 short results give 2 calls: one of 32 results and one of 1 result.
//! Standard output is a pipe in each test.

mod common;

use std::path::Path;
use std::process::Output;
use std::sync::Mutex;

use common::fake_server::{FakeServer, Reply, Request};
use common::{jag, jag_std, write};
use serde_json::{Value, json};
use tempfile::TempDir;

const QUESTION: &str = "how long is the retry wait";

/// `count` files with one matching line each.
fn folder(count: usize) -> TempDir {
    let root = tempfile::tempdir().expect("folder");
    for number in 0..count {
        write(root.path(), &format!("dir/f{number:03}.txt"), b"needle\n");
    }
    root
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn document(&self) -> Value {
        serde_json::from_str(&self.stdout).expect("a document")
    }
}

fn finish(output: Output) -> Run {
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// `jag [extra] [--ask QUESTION] needle dir`, with `environment`.
fn run(server: &FakeServer, root: &Path, extra: &[&str], environment: &[(&str, &str)]) -> Run {
    run_with(server, root, extra, environment, true)
}

fn run_with(
    server: &FakeServer,
    root: &Path,
    extra: &[&str],
    environment: &[(&str, &str)],
    ask: bool,
) -> Run {
    let home = tempfile::tempdir().expect("home");
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in environment {
        command.env(name, value);
    }
    // The long temporary paths of macOS can make the results larger than the
    // default size limit.
    command.args(["--no-max-chars"]).args(extra);
    if ask {
        command.args(["--ask", QUESTION]);
    }
    command.args(["needle", "dir"]).current_dir(root);
    finish(command.output().expect("run jag"))
}

/// A server that gives `tokens` input tokens for each call.
fn tokens_server(tokens: u64) -> FakeServer {
    FakeServer::start(move |request| {
        Reply::scores_with_usage(
            request,
            |_| 0.8,
            Some(json!({"input_tokens": tokens, "output_tokens": 1})),
        )
    })
}

/// A server that gives each call the value of `usage`. `None` leaves the
/// field out.
fn usage_server(usage: Option<Value>) -> FakeServer {
    FakeServer::start(move |request| Reply::scores_with_usage(request, |_| 0.8, usage.clone()))
}

/// The amount of a number of billionths, rounded to millionths.
fn dollars(billionths: u64) -> String {
    let millionths = (billionths + 500) / 1000;
    format!("${}.{:06}", millionths / 1_000_000, millionths % 1_000_000)
}

fn line(amount: &str, scored: usize, found: usize) -> String {
    format!(
        "jag: cost {amount}: scored {scored} of {found} results, skipped {}.\n",
        found - scored
    )
}

const COST_LIMIT_NOTICE: &str = "jag: cost limit reached: scored 1 of 2 results, skipped 1.\n";

// CRITERION-88: one cost line after each search, also in a pipe.
#[test]
fn criterion_88_a_text_search_writes_one_cost_line() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    let run = run(&server, root.path(), &[], &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, line("$0.000000", 2, 2));
}

#[test]
fn criterion_88_the_cost_line_comes_after_the_cost_limit_notice() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    let run = run(&server, root.path(), &[], &[("JAG_MAX_SCORED", "1")]);
    assert_eq!(run.code, Some(7));
    assert_eq!(
        run.stderr,
        format!("{COST_LIMIT_NOTICE}{}", line("$0.000000", 1, 2))
    );
}

#[test]
fn criterion_88_plain_grep_writes_the_cost_line() {
    let server = FakeServer::constant(0.8);
    let root = folder(3);
    let run = run_with(&server, root.path(), &[], &[], false);
    assert_eq!(run.code, Some(0));
    assert_eq!(
        run.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.\n"
    );
}

#[test]
fn criterion_88_a_search_with_json_writes_nothing_on_the_error_stream() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    assert_eq!(run(&server, root.path(), &["--json"], &[]).stderr, "");
    assert_eq!(
        run_with(&server, root.path(), &["--json"], &[], false).stderr,
        ""
    );
}

// CRITERION-89: the counts and the billed amount.
#[test]
fn criterion_89_the_amount_is_the_input_tokens_at_the_price() {
    // 48,000 tokens times 42 is 2,016,000 billionths.
    let server = tokens_server(24_000);
    let root = folder(33);
    let run = run(&server, root.path(), &[], &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(server.requests().len(), 2);
    assert_eq!(run.stderr, line("$0.002016", 33, 33));
}

#[test]
fn criterion_89_a_search_with_skipped_results_counts_one_call() {
    let server = tokens_server(24_000);
    let root = folder(33);
    let run = run(&server, root.path(), &[], &[("JAG_MAX_SCORED", "32")]);
    assert_eq!(run.code, Some(7));
    assert_eq!(server.requests().len(), 1);
    assert_eq!(
        run.stderr,
        format!(
            "jag: cost limit reached: scored 32 of 33 results, skipped 1.\n{}",
            line("$0.001008", 32, 33)
        )
    );
}

#[test]
fn criterion_89_a_retry_counts_one_time_and_the_failed_attempt_adds_nothing() {
    let seen = Mutex::new(0);
    let server = FakeServer::start(move |request| {
        let mut seen = seen.lock().expect("lock");
        *seen += 1;
        if *seen == 1 {
            Reply::status(503)
        } else {
            Reply::scores_with_usage(request, |_| 0.8, Some(json!({"input_tokens": 1000})))
        }
    });
    let root = folder(1);
    let run = run(&server, root.path(), &["--retry-wait", "1"], &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(server.requests().len(), 2);
    assert_eq!(run.stderr, line("$0.000042", 1, 1));
}

// CRITERION-90: the same counts and amount in the field.
/// The cost line of a text search, as numbers: the amount in millionths or
/// `None`, the kind words, the reason, and the three counts.
fn parse_line(stderr: &str) -> (Option<u64>, bool, Option<String>, [u64; 3]) {
    let last = stderr.trim_end().lines().last().expect("a line");
    let text = last.strip_prefix("jag: cost ").expect("a cost line");
    let (amount, counts) = text.split_once(": scored ").expect("counts");
    let numbers: Vec<u64> = counts
        .split(|c: char| !c.is_ascii_digit())
        .filter(|word| !word.is_empty())
        .map(|word| word.parse().expect("number"))
        .collect();
    let (micro, estimate, reason) = if amount == "not available" {
        (None, false, None)
    } else {
        let (money, rest) = amount.split_once(' ').unwrap_or((amount, ""));
        let micro = money.trim_start_matches('$').replace('.', "");
        let reason = rest
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
            .filter(|word| *word != "estimate")
            .map(|word| word.replace(' ', "_"));
        (
            Some(micro.parse().expect("amount")),
            rest == "(estimate)",
            reason,
        )
    };
    (
        micro,
        estimate,
        reason,
        [numbers[0], numbers[1], numbers[2]],
    )
}

fn assert_same(text: &Run, json: &Run, kind: &str) {
    let (micro, estimate, reason, [scored, found, skipped]) = parse_line(&text.stderr);
    let document = json.document();
    let cost = &document["cost"];
    assert_eq!(cost["micro_usd"], json!(micro));
    assert_eq!(cost["kind"], kind);
    assert_eq!(estimate, kind == "estimate");
    assert_eq!(cost["reason"], json!(reason));
    assert_eq!(document["results_scored"], scored);
    assert_eq!(document["results_found"], found);
    assert_eq!(document["results_skipped"], skipped);
    assert_eq!(json.stderr, "");
}

#[test]
fn criterion_90_the_field_and_the_line_hold_the_same_values() {
    let root = folder(2);
    let args = ["--json"];

    let server = tokens_server(24_000);
    let text = run(&server, root.path(), &[], &[]);
    assert_same(&text, &run(&server, root.path(), &args, &[]), "billed");

    let server = usage_server(None);
    let text = run(&server, root.path(), &[], &[]);
    assert_same(&text, &run(&server, root.path(), &args, &[]), "estimate");

    let server = FakeServer::constant(0.8);
    let model = [("JAG_MODEL", "jev-9.9.9")];
    let text = run(&server, root.path(), &[], &model);
    assert_same(
        &text,
        &run(&server, root.path(), &args, &model),
        "not_available",
    );

    let none = |extra: &[&str]| {
        let home = tempfile::tempdir().expect("home");
        finish(
            jag(home.path(), &server.base_url())
                .args(extra)
                .args(["--ask", QUESTION, "absent", "dir"])
                .current_dir(root.path())
                .output()
                .expect("run jag"),
        )
    };
    assert_same(&none(&[]), &none(&args), "no_call");

    let plain = |extra: &[&str]| run_with(&server, root.path(), extra, &[], false);
    assert_same(&plain(&[]), &plain(&args), "no_call");
}

// CRITERION-91: zero and the reason when there is no call.
#[test]
fn criterion_91_a_term_that_matches_nothing_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    let home = tempfile::tempdir().expect("home");
    let ask = |extra: &[&str]| {
        finish(
            jag(home.path(), &server.base_url())
                .args(extra)
                .args(["--ask", QUESTION, "absent", "dir"])
                .current_dir(root.path())
                .output()
                .expect("run jag"),
        )
    };
    let text = ask(&[]);
    assert_eq!(text.code, Some(1));
    assert_eq!(
        text.stderr,
        "jag: cost $0.000000 (no match): scored 0 of 0 results, skipped 0.\n"
    );
    let json = ask(&["--json"]);
    assert_eq!(
        json.document()["cost"],
        json!({"micro_usd": 0, "kind": "no_call", "reason": "no_match"})
    );
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_91_plain_grep_makes_no_call_with_a_match_and_with_none() {
    let server = FakeServer::constant(0.8);
    let root = folder(3);
    let home = tempfile::tempdir().expect("home");
    let grep = |term: &str, extra: &[&str]| {
        finish(
            jag(home.path(), &server.base_url())
                .args(extra)
                .args([term, "dir"])
                .current_dir(root.path())
                .output()
                .expect("run jag"),
        )
    };
    let hit = grep("needle", &[]);
    assert_eq!(hit.code, Some(0));
    assert_eq!(
        hit.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 3 results, skipped 0.\n"
    );
    let miss = grep("absent", &[]);
    assert_eq!(miss.code, Some(1));
    assert_eq!(
        miss.stderr,
        "jag: cost $0.000000 (no question): scored 0 of 0 results, skipped 0.\n"
    );
    for term in ["needle", "absent"] {
        let json = grep(term, &["--json"]);
        let document = json.document();
        assert_eq!(
            document["cost"],
            json!({"micro_usd": 0, "kind": "no_call", "reason": "no_question"})
        );
        assert_eq!(document["results_scored"], 0);
        assert_eq!(document["results_skipped"], 0);
    }
    assert!(server.requests().is_empty());
}

// CRITERION-92: not available, and the search does not fail.
#[test]
fn criterion_92_a_model_with_no_price_gives_not_available() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    let pinned = run(&server, root.path(), &[], &[]);
    let model = [("JAG_MODEL", "jev-9.9.9")];
    let other = run(&server, root.path(), &[], &model);
    assert_eq!(other.code, Some(0));
    assert_eq!(other.stdout, pinned.stdout);
    assert_eq!(
        other.stderr,
        "jag: cost not available: scored 2 of 2 results, skipped 0.\n"
    );
    let json = run(&server, root.path(), &["--json"], &model);
    let cost = &json.document()["cost"];
    assert_eq!(cost["micro_usd"], Value::Null);
    assert_eq!(cost["kind"], "not_available");
}

// CRITERION-97: an estimate with a label.
/// The estimate of all requests that the server got: the bytes of each body
/// divided by 3 (rounded down), at 42 billionths for each token. `known` is
/// the tokens that the service gave.
fn estimate_of(requests: &[Request], known: u64) -> String {
    let tokens: u64 = requests
        .iter()
        .map(|request| request.body_text().len() as u64 / 3)
        .sum();
    dollars((tokens + known) * 42)
}

#[test]
fn criterion_97_an_answer_with_no_count_gives_an_estimate() {
    let root = folder(2);
    for usage in [
        None,
        Some(json!(null)),
        Some(json!({})),
        Some(json!({"input_tokens": null})),
        Some(json!({"input_tokens": "12"})),
        Some(json!({"input_tokens": -1})),
        Some(json!({"input_tokens": 2.5})),
    ] {
        let server = usage_server(usage.clone());
        let text = run(&server, root.path(), &[], &[]);
        assert_eq!(text.code, Some(0), "{usage:?}");
        let amount = format!("{} (estimate)", estimate_of(&server.requests(), 0));
        assert_eq!(text.stderr, line(&amount, 2, 2), "{usage:?}");
        let json = run(&server, root.path(), &["--json"], &[]);
        assert_eq!(json.document()["cost"]["kind"], "estimate", "{usage:?}");
    }
}

#[test]
fn criterion_97_one_call_with_no_count_makes_the_whole_amount_an_estimate() {
    let server = FakeServer::start(|request| {
        let usage = (request.snippet_paths().len() == 32)
            .then(|| json!({"input_tokens": 24_000, "output_tokens": 1}));
        Reply::scores_with_usage(request, |_| 0.8, usage)
    });
    let root = folder(33);
    let text = run(&server, root.path(), &[], &[]);
    assert_eq!(text.code, Some(0));
    let requests = server.requests();
    assert_eq!(requests.len(), 2);
    let small: Vec<Request> = requests
        .into_iter()
        .filter(|request| request.snippet_paths().len() == 1)
        .collect();
    let amount = format!("{} (estimate)", estimate_of(&small, 24_000));
    assert_eq!(text.stderr, line(&amount, 33, 33));
}

// CRITERION-93: the cost of a failed search.
#[test]
fn criterion_93_a_failure_gives_the_cost_up_to_that_point() {
    let server = FakeServer::start(|request| {
        if request.snippet_paths().len() == 32 {
            Reply::scores_with_usage(request, |_| 0.8, Some(json!({"input_tokens": 24_000})))
        } else {
            Reply::status(500)
        }
    });
    let root = folder(33);
    let args = ["--retry-wait", "1"];
    let text = run(&server, root.path(), &args, &[]);
    assert_eq!(text.code, Some(6));
    assert_eq!(
        text.stderr,
        format!(
            "jag: the gateway failed (HTTP 500).\nTry again later.\n{}",
            line("$0.001008", 33, 33)
        )
    );
    let json = run(&server, root.path(), &["--json", "--retry-wait", "1"], &[]);
    assert_eq!(json.code, Some(6));
    assert_eq!(
        json.document()["cost"],
        json!({"micro_usd": 1008, "kind": "billed", "reason": null})
    );
    assert_eq!(json.stderr, "");
}

#[test]
fn criterion_93_a_key_that_is_not_accepted_costs_nothing() {
    let server = FakeServer::start(|_| Reply::status(401));
    let root = folder(2);
    let text = run(&server, root.path(), &[], &[]);
    assert_eq!(text.code, Some(4));
    assert!(
        text.stderr.ends_with(&line("$0.000000", 2, 2)),
        "{}",
        text.stderr
    );
}

#[cfg(target_os = "linux")]
#[test]
fn criterion_93_a_write_failure_gives_the_text_failure_and_the_cost_line() {
    let server = FakeServer::constant(0.8);
    let root = folder(2);
    for json in [false, true] {
        let home = tempfile::tempdir().expect("home");
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("open /dev/full");
        let mut command = jag_std(home.path(), &server.base_url());
        command
            .args(["--no-max-chars"])
            .args(if json { &["--json"][..] } else { &[] })
            .args(["--ask", QUESTION, "needle", "dir"])
            .current_dir(root.path())
            .stdout(std::process::Stdio::from(full));
        let run = finish(command.output().expect("run jag"));
        assert_eq!(run.code, Some(2), "{json}");
        assert_eq!(
            run.stderr,
            format!(
                "jag: jag could not write the results.\nCheck where the output goes.\n{}",
                line("$0.000000", 2, 2)
            ),
            "{json}"
        );
    }
}
