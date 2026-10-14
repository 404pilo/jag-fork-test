//! End-to-end tests of FEATURE-06: the BM25 pre-rank, the limit setting, the
//! skip notice, and exit 7 (CRITERION-25 to CRITERION-28). They use the fake
//! server only.

mod common;

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Output;

use common::fake_server::{FakeServer, Reply};
use common::{jag, write};
use jag::output::NO_GOOD_ANSWER;
use tempfile::TempDir;

const QUESTION: &str = "how long is the retry wait";

/// 250 files `f000.txt` to `f249.txt` with `needle`, and 5 files `z0.txt` to
/// `z4.txt` that hold the question words. N = 255. The `z` files come last in
/// path order.
fn large_folder() -> TempDir {
    let dir = tempfile::tempdir().expect("temporary folder");
    for number in 0..250 {
        write(dir.path(), &format!("f{number:03}.txt"), b"needle\n");
    }
    write(dir.path(), "z0.txt", b"needle retry\n");
    for number in 1..5 {
        write(
            dir.path(),
            &format!("z{number}.txt"),
            b"needle retry wait\n",
        );
    }
    dir
}

fn small_folder(files: usize) -> TempDir {
    let dir = tempfile::tempdir().expect("temporary folder");
    for number in 0..files {
        write(dir.path(), &format!("f{number:03}.txt"), b"needle\n");
    }
    dir
}

fn search(server: &FakeServer, dir: &Path, extra: &[&str], environment: &[(&str, &str)]) -> Output {
    let home = tempfile::tempdir().expect("temporary folder");
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command
        .args(extra)
        // The tests here are about the cost limit, not the size limit: the
        // long temporary paths of macOS can make 200 results larger than the
        // default size limit.
        .args(["--no-max-chars", "--ask", QUESTION, "needle"])
        .arg(dir)
        .output()
        .expect("run jag");
    for text in [stdout(&output), stderr(&output)] {
        assert!(!text.contains("SENTINEL"), "{text}");
    }
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The count of snippets that all requests hold.
fn sent(server: &FakeServer) -> usize {
    server
        .requests()
        .iter()
        .map(|request| request.snippet_texts().len())
        .sum()
}

/// The file names of the snippets that all requests hold.
fn sent_names(server: &FakeServer) -> BTreeSet<String> {
    server
        .requests()
        .iter()
        .flat_map(|request| request.snippet_paths())
        .map(|path| path.rsplit('/').next().unwrap_or_default().to_owned())
        .collect()
}

/// The count of results on standard output (one match line each).
fn results(output: &Output) -> usize {
    stdout(output)
        .lines()
        .filter(|line| line.contains(":1:needle"))
        .count()
}

fn notice(scored: usize, total: usize) -> String {
    format!(
        "jag: cost limit reached: scored {scored} of {total} results, skipped {}.",
        total - scored
    )
}

// CRITERION-25: jag scores only the best matches.
#[test]
fn the_default_limit_scores_the_200_best_results() {
    let server = FakeServer::constant(0.8);
    let dir = large_folder();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(sent(&server), 200);
    let names = sent_names(&server);
    for z in ["z0.txt", "z1.txt", "z2.txt", "z3.txt", "z4.txt"] {
        assert!(names.contains(z), "{z} was not sent");
    }
    assert_eq!(results(&output), 200);
}

#[test]
fn a_small_limit_sends_the_best_results_in_path_order() {
    let server = FakeServer::constant(0.8);
    let dir = large_folder();
    let output = search(&server, dir.path(), &["--max-scored", "5"], &[]);
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let names: Vec<String> = requests[0]
        .snippet_paths()
        .iter()
        .map(|path| path.rsplit('/').next().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(names, ["z0.txt", "z1.txt", "z2.txt", "z3.txt", "z4.txt"]);
    assert_eq!(results(&output), 5);
}

// CRITERION-26: the limit has a default and the user can change it.
#[test]
fn the_user_can_change_the_limit() {
    type Case<'a> = (&'a [&'a str], &'a [(&'a str, &'a str)], usize);
    let cases: [Case; 6] = [
        (&[], &[], 200),
        (&["--max-scored", "10"], &[], 10),
        (&[], &[("JAG_MAX_SCORED", "10")], 10),
        (&["--max-scored", "10"], &[("JAG_MAX_SCORED", "20")], 10),
        (&[], &[("JAG_MAX_SCORED", "")], 200),
        (&["--max-scored", "2000"], &[], 255),
    ];
    for (extra, environment, count) in cases {
        let server = FakeServer::constant(0.8);
        let dir = large_folder();
        search(&server, dir.path(), extra, environment);
        assert_eq!(sent(&server), count, "{extra:?} {environment:?}");
    }
}

#[test]
fn a_limit_of_2000_scores_all_255_results_and_gives_no_notice() {
    let server = FakeServer::constant(0.8);
    let dir = large_folder();
    let output = search(&server, dir.path(), &["--max-scored", "2000"], &[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!stderr(&output).contains("cost limit"));
}

// CRITERION-27: jag says how many matches it skipped.
#[test]
fn the_notice_says_how_many_results_jag_skipped() {
    let server = FakeServer::constant(0.8);
    let dir = large_folder();
    let output = search(&server, dir.path(), &[], &[]);
    assert!(
        stderr(&output).contains(&notice(200, 255)),
        "{}",
        stderr(&output)
    );
    assert_eq!(results(&output), 200);

    let server = FakeServer::constant(0.8);
    let output = search(&server, dir.path(), &["--max-scored", "5"], &[]);
    assert!(
        stderr(&output).contains(&notice(5, 255)),
        "{}",
        stderr(&output)
    );
    assert_eq!(results(&output), 5);
}

#[test]
fn results_that_are_not_more_than_the_limit_give_no_notice() {
    for files in [199, 200] {
        let server = FakeServer::constant(0.8);
        let dir = small_folder(files);
        let output = search(&server, dir.path(), &[], &[]);
        assert!(!stderr(&output).contains("cost limit"), "{files} files");
        // CRITERION-28: not more than the limit gives exit 0.
        assert_eq!(output.status.code(), Some(0), "{files} files");
        assert_eq!(sent(&server), files);
    }
}

// CRITERION-28: reaching the limit has its own exit code.
#[test]
fn reaching_the_limit_gives_exit_7() {
    let server = FakeServer::constant(0.8);
    let dir = large_folder();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn exit_1_wins_over_exit_7_and_the_notice_still_shows() {
    let server = FakeServer::constant(0.2);
    let dir = large_folder();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output).lines().next(), Some(NO_GOOD_ANSWER));
    assert!(
        stderr(&output).contains(&notice(200, 255)),
        "{}",
        stderr(&output)
    );
}

// A model failure gives its own code and no result, with the limit too.
#[test]
fn a_model_failure_with_the_limit_gives_exit_6_and_the_notice() {
    let server = FakeServer::start(|_| Reply::status(500));
    let dir = large_folder();
    let output = search(&server, dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(6));
    assert!(stdout(&output).is_empty());
    let err = stderr(&output);
    assert!(err.contains(&notice(200, 255)), "{err}");
    assert!(err.contains("the TypeSafe API failed (HTTP 500)."), "{err}");
}

#[test]
fn two_runs_send_the_same_requests() {
    let dir = large_folder();
    let mut runs = Vec::new();
    for _ in 0..2 {
        let server = FakeServer::constant(0.8);
        search(&server, dir.path(), &["--max-scored", "40"], &[]);
        // The calls run in parallel, so compare the requests as a set.
        let bodies: BTreeSet<String> = server
            .requests()
            .iter()
            .map(|request| request.body_text())
            .collect();
        runs.push(bodies);
    }
    assert_eq!(runs[0], runs[1]);
}
