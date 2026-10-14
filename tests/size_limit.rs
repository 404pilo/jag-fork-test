//! End-to-end tests of FEATURE-09: the size limit on standard output
//! (CRITERION-36 to CRITERION-42, CRITERION-66, and CRITERION-104). They use
//! the fake server only.
//!
//! Each test folder holds its files in a folder `d`, and jag runs with
//! `current_dir` in the test folder and the path `d`, so each path is
//! `d/<name>` and each count is the same on each machine.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;

use common::fake_server::{FakeServer, Reply};
use common::{jag, jag_bare, write};
use serde_json::Value;
use tempfile::TempDir;

const FIX_1000: &str =
    "To see more, use --max-chars with a number larger than 1000, or --no-max-chars.";

fn folder() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

/// The "ranked folder": 30 files `r00.txt` to `r29.txt`, with one line of 100
/// characters each. The server gives `rNN.txt` the score `0.9 - NN / 100`.
fn ranked_folder() -> TempDir {
    let root = folder();
    for number in 0..30 {
        let head = format!("needle {number:02} ");
        let line = format!("{head}{}\n", "x".repeat(100 - head.len()));
        write(root.path(), &format!("d/r{number:02}.txt"), line.as_bytes());
    }
    root
}

/// The server of the ranked folder.
fn ranked_server() -> FakeServer {
    FakeServer::start(|request| {
        Reply::scores(request, |text| {
            let at = text.find("needle ").map_or(0, |at| at + "needle ".len());
            let number: f64 = text[at..at + 2].parse().unwrap_or(0.0);
            0.9 - number / 100.0
        })
    })
}

/// The "big folder": 200 files with one line of 100 characters each.
fn big_folder() -> TempDir {
    let root = folder();
    for number in 0..200 {
        let head = "needle ";
        let line = format!("{head}{}\n", "x".repeat(100 - head.len()));
        write(root.path(), &format!("d/b{number:03}.txt"), line.as_bytes());
    }
    root
}

/// The "UTF-8 folder": 10 files with `needle ` and 93 `é` characters.
fn utf8_folder() -> TempDir {
    let root = folder();
    for number in 0..10 {
        let line = format!("needle {}\n", "é".repeat(93));
        write(root.path(), &format!("d/u{number}.txt"), line.as_bytes());
    }
    root
}

/// The folder of Example 1: `top.txt` has 7 lines of 200 characters, and
/// line 4 matches. `low.txt` has the line `needle`.
fn cut_folder(with_low: bool) -> TempDir {
    let root = folder();
    let mut top = String::new();
    for number in 1..=7 {
        let head = if number == 4 {
            "l4 needle ".to_owned()
        } else {
            format!("l{number} ")
        };
        top.push_str(&format!("{head}{}\n", "x".repeat(200 - head.len())));
    }
    write(root.path(), "d/top.txt", top.as_bytes());
    if with_low {
        write(root.path(), "d/low.txt", b"needle\n");
    }
    root
}

fn cut_server() -> FakeServer {
    FakeServer::start(|request| {
        Reply::scores(
            request,
            |text| {
                if text.contains("l4 needle") { 0.9 } else { 0.5 }
            },
        )
    })
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn chars(&self) -> usize {
        self.stdout.chars().count()
    }
}

fn run_of(output: Output) -> Run {
    let run = Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    for text in [&run.stdout, &run.stderr] {
        assert!(!text.contains("SENTINEL"), "{text}");
    }
    run
}

/// Run `jag ARGS d` in `cwd`, with the key and the fake server.
fn run(server: &FakeServer, cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> Run {
    let home = folder();
    let mut command = jag(home.path(), &server.base_url());
    for (name, value) in env {
        command.env(name, value);
    }
    run_of(
        command
            .current_dir(cwd)
            .args(args)
            .arg("d")
            .output()
            .expect("run jag"),
    )
}

/// A search with the question `q` and the term `needle`.
fn ask(server: &FakeServer, cwd: &Path, extra: &[&str], env: &[(&str, &str)]) -> Run {
    let mut args = vec!["--ask", "q", "needle"];
    args.splice(0..0, extra.iter().copied());
    run(server, cwd, &args, env)
}

fn notice(shown: usize, total: usize, limit: &str) -> String {
    format!(
        "jag: size limit reached: showed {shown} of {total} results, left out {}. \
         To see more, use --max-chars with a number larger than {limit}, or --no-max-chars.\n",
        total - shown
    )
}

/// The names `rNN` that standard output shows, in order.
fn shown_names(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.starts_with("d/"))
        .map(|line| line[2..line.find(".txt").unwrap_or(line.len())].to_owned())
        .collect()
}

fn names(from: usize, to: usize) -> Vec<String> {
    (from..=to).map(|number| format!("r{number:02}")).collect()
}

// CRITERION-36, CRITERION-39, CRITERION-41, CRITERION-42: whole results, the
// lowest rank first, one notice, exit 8.
#[test]
fn criterion_36_39_41_42_the_ranked_folder_with_a_limit_of_1000() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = ask(&server, root.path(), &["--max-chars", "1000"], &[]);
    assert_eq!(run.chars(), 925);
    assert_eq!(shown_names(&run.stdout), names(0, 7));
    assert_eq!(run.stderr, notice(8, 30, "1000"));
    assert_eq!(run.code, Some(8));
    // Each result that shows has all its lines.
    for line in run.stdout.lines().filter(|line| *line != "--") {
        assert_eq!(line.chars().count(), 112, "{line}");
    }
}

#[test]
fn criterion_36_the_limit_is_a_count_that_the_output_can_reach() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = ask(&server, root.path(), &["--max-chars", "1041"], &[]);
    assert_eq!(run.chars(), 1041);
    assert_eq!(shown_names(&run.stdout), names(0, 8));
    assert_eq!(run.code, Some(8));
}

#[test]
fn criterion_36_the_limit_counts_characters_and_not_bytes() {
    let server = FakeServer::constant(0.8);
    let root = utf8_folder();
    let run = ask(&server, root.path(), &["--max-chars", "1147"], &[]);
    assert_eq!(run.chars(), 1147);
    assert!(run.stdout.len() > 2000);
    assert_eq!(run.stdout.matches(":1:needle").count(), 10);
    assert_eq!(run.stderr, "");
    assert_eq!(run.code, Some(0));
    let run = ask(&server, root.path(), &["--max-chars", "1146"], &[]);
    assert_eq!(run.stdout.matches(":1:needle").count(), 9);
    assert_eq!(run.code, Some(8));
}

// CRITERION-37: a default limit always applies.
#[test]
fn criterion_37_the_default_limit_is_20000() {
    let server = FakeServer::constant(0.8);
    let root = big_folder();
    let run = ask(&server, root.path(), &[], &[]);
    assert!(
        run.chars() <= 20_000 && run.chars() > 19_000,
        "{}",
        run.chars()
    );
    assert_eq!(run.code, Some(8));
    assert!(
        run.stderr
            .trim_end()
            .ends_with("larger than 20000, or --no-max-chars."),
        "{}",
        run.stderr
    );
}

#[test]
fn criterion_37_the_user_can_raise_the_limit_or_turn_it_off() {
    let server = FakeServer::constant(0.8);
    let root = big_folder();
    for (args, env) in [
        (vec!["--max-chars", "40000"], vec![]),
        (vec!["--no-max-chars"], vec![]),
        (vec![], vec![("JAG_MAX_CHARS", "40000")]),
        (vec!["--no-max-chars"], vec![("JAG_MAX_CHARS", "abc")]),
    ] {
        let run = ask(&server, root.path(), &args, &env);
        assert_eq!(run.stdout.matches(":1:needle").count(), 200, "{args:?}");
        assert_eq!(run.code, Some(0), "{args:?}");
        assert_eq!(run.stderr, "", "{args:?}");
    }
}

#[test]
fn criterion_37_the_option_wins_over_the_variable_and_a_blank_variable_is_the_default() {
    let server = ranked_server();
    let root = ranked_folder();
    let env = [("JAG_MAX_CHARS", "40000")];
    let run = ask(&server, root.path(), &["--max-chars", "1000"], &env);
    assert!(run.chars() <= 1000);
    assert_eq!(run.code, Some(8));
    let run = ask(&server, root.path(), &[], &[("JAG_MAX_CHARS", "1000")]);
    assert_eq!(run.chars(), 925);
    // A blank variable is the default of 20,000: the 30 results show.
    let run = ask(&server, root.path(), &[], &[("JAG_MAX_CHARS", "")]);
    assert_eq!(shown_names(&run.stdout).len(), 30);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, "");
}

// CRITERION-38: all text on standard output counts, and the error stream does
// not.
#[test]
fn criterion_38_the_label_and_the_separators_count() {
    let server = FakeServer::constant(0.2);
    let root = ranked_folder();
    // The ranked folder with a low score for each snippet.
    let run = ask(&server, root.path(), &["--max-chars", "1041"], &[]);
    assert!(run.stdout.starts_with("jag: no good answer."));
    assert_eq!(run.chars(), 997);
    assert_eq!(run.stdout.matches(":1:needle").count(), 8);
    assert_eq!(run.code, Some(1));
    assert!(run.stderr.contains("showed 8 of 30 results"));
}

// CRITERION-40: the top result is cut when it alone is larger than the limit.
#[test]
fn criterion_40_the_top_result_is_cut_and_the_notice_says_so() {
    let server = cut_server();
    let root = cut_folder(true);
    let run = ask(&server, root.path(), &["--max-chars", "1000"], &[]);
    assert_eq!(run.chars(), 865);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].starts_with("d/top.txt-1-l1 "), "{}", lines[0]);
    assert!(lines[3].starts_with("d/top.txt:4:l4 needle "));
    assert!(lines[3].ends_with(" [+603 chars]"), "{}", lines[3]);
    assert!(!run.stdout.contains("low.txt"));
    assert_eq!(run.code, Some(8));
    assert_eq!(
        run.stderr,
        format!(
            "jag: size limit reached: showed 1 of 2 results, left out 1, and cut the top result. {FIX_1000}\n"
        )
    );
}

#[test]
fn criterion_40_a_cut_with_no_other_result_gives_exit_8() {
    let server = cut_server();
    let root = cut_folder(false);
    let run = ask(&server, root.path(), &["--max-chars", "1000"], &[]);
    assert_eq!(run.chars(), 865);
    assert_eq!(run.code, Some(8));
    assert_eq!(
        run.stderr,
        format!(
            "jag: size limit reached: showed 1 of 1 results, left out 0, and cut the top result. {FIX_1000}\n"
        )
    );
}

#[test]
fn a_limit_that_the_top_result_fits_in_gives_no_cut() {
    let server = cut_server();
    let root = cut_folder(true);
    let run = ask(&server, root.path(), &["--max-chars", "4000"], &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, "");
    assert!(!run.stdout.contains("chars]"));
}

// CRITERION-41: no setting gives no size notice.
#[test]
fn criterion_41_a_search_inside_the_limit_gives_no_notice() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = ask(&server, root.path(), &[], &[]);
    assert!(!run.stderr.contains("size limit"));
    assert_eq!(run.code, Some(0));
    let run = ask(&server, root.path(), &["--max-chars", "4000"], &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stderr, "");
}

// CRITERION-66: the cost-limit code wins, and the notice still says so.
#[test]
fn criterion_66_the_cost_limit_code_wins_and_both_notices_show() {
    let server = ranked_server();
    let root = ranked_folder();
    let args = ["--max-scored", "10", "--max-chars", "1000"];
    let run = ask(&server, root.path(), &args, &[]);
    assert_eq!(run.code, Some(7));
    assert_eq!(
        run.stderr,
        format!(
            "jag: cost limit reached: scored 10 of 30 results, skipped 20.\n{}",
            notice(8, 10, "1000")
        )
    );
    assert_eq!(run.stdout.matches(":1:needle").count(), 8);
}

// A model failure gives its own code and no size notice.
#[test]
fn a_model_failure_gives_no_size_notice() {
    let server = FakeServer::start(|_| Reply::status(500));
    let root = ranked_folder();
    let run = ask(&server, root.path(), &["--max-chars", "1000"], &[]);
    assert_eq!(run.code, Some(6));
    assert_eq!(run.stdout, "");
    assert!(!run.stderr.contains("size limit"));
}

fn parse_doc(run: &Run) -> Value {
    assert!(run.stdout.ends_with('\n'));
    assert_eq!(run.stdout.matches('\n').count(), 1);
    serde_json::from_str(&run.stdout).expect("a JSON document")
}

fn result_names(document: &Value) -> Vec<String> {
    document["results"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|result| {
            let path = result["path"].as_str().unwrap_or_default();
            path[2..path.find(".txt").unwrap_or(path.len())].to_owned()
        })
        .collect()
}

// CRITERION-104: the size-limit notice is named fields in structured output.
#[test]
fn criterion_104_the_document_has_the_fields_of_the_notice() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = ask(
        &server,
        root.path(),
        &["--json", "--max-chars", "1000"],
        &[],
    );
    let document = parse_doc(&run);
    assert_eq!(run.chars(), 823);
    assert_eq!(result_names(&document), names(0, 1));
    assert_eq!(document["left_out"], 28);
    assert_eq!(document["top_result_cut"], 0);
    assert_eq!(document["warnings_left_out"], 0);
    assert_eq!(document["size_limit_fix"], FIX_1000);
    assert_eq!(document["exit_code"], 8);
    assert_eq!(run.code, Some(8));
    assert_eq!(run.stderr, "");
}

#[test]
fn criterion_104_a_document_inside_the_limit_has_the_values_of_nothing_left_out() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = ask(&server, root.path(), &["--json"], &[]);
    let document = parse_doc(&run);
    assert_eq!(run.chars(), 6812);
    assert_eq!(document["results"].as_array().map(Vec::len), Some(30));
    assert_eq!(document["left_out"], 0);
    assert_eq!(document["top_result_cut"], 0);
    assert_eq!(document["size_limit_fix"], "");
    assert_eq!(document["warnings_left_out"], 0);
    assert_eq!(run.code, Some(0));
}

#[test]
fn criterion_104_the_document_never_goes_over_the_limit_and_left_out_is_n_minus_k() {
    let server = ranked_server();
    let root = ranked_folder();
    // The count of results in the document, and its size, for one limit.
    let at = |limit: usize| {
        let text = limit.to_string();
        let run = ask(&server, root.path(), &["--json", "--max-chars", &text], &[]);
        let document = parse_doc(&run);
        let shown = document["results"].as_array().map_or(0, Vec::len);
        (run, document, shown)
    };
    for limit in (1000..=7000).step_by(1000) {
        let (run, document, shown) = at(limit);
        assert!(run.chars() <= limit, "{limit}: {}", run.chars());
        assert_eq!(document["left_out"], 30 - shown, "{limit}");
        assert_eq!(run.code, Some(if shown < 30 { 8 } else { 0 }), "{limit}");
        if shown == 30 {
            continue;
        }
        // K is the largest count: the smallest limit that gives one more
        // result gives a document that is larger than this limit.
        let (mut low, mut high) = (limit, 8000);
        while high - low > 1 {
            let middle = (low + high) / 2;
            if at(middle).2 > shown {
                high = middle;
            } else {
                low = middle;
            }
        }
        let (bigger, _, more) = at(high);
        assert_eq!(more, shown + 1, "{limit}");
        assert!(bigger.chars() > limit, "{limit}: {}", bigger.chars());
    }
}

#[test]
fn criterion_104_the_top_result_is_cut_in_a_document() {
    let server = cut_server();
    let root = cut_folder(true);
    let run = ask(
        &server,
        root.path(),
        &["--json", "--max-chars", "1000"],
        &[],
    );
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000);
    assert_eq!(document["left_out"], 1);
    assert!(document["top_result_cut"].as_u64().unwrap_or(0) > 0);
    assert_eq!(document["size_limit_fix"], FIX_1000);
    assert_eq!(document["exit_code"], 8);
    assert_eq!(run.code, Some(8));
    let result = &document["results"][0];
    assert_eq!(result["path"], "d/top.txt");
    assert!(result["lines"].as_array().map_or(0, Vec::len) >= 1);
    assert!(!run.stdout.contains("chars]"));
    assert_eq!(run.stderr, "");

    let root = cut_folder(false);
    let run = ask(
        &server,
        root.path(),
        &["--json", "--max-chars", "1000"],
        &[],
    );
    let document = parse_doc(&run);
    assert_eq!(document["left_out"], 0);
    assert!(document["top_result_cut"].as_u64().unwrap_or(0) > 0);
    assert_eq!(document["exit_code"], 8);
    assert_eq!(run.code, Some(8));
}

/// The folder of 50 files that jag cannot read, and one file that matches.
/// Gives false when the user can read the files anyway (root).
fn locked_folder() -> (TempDir, bool) {
    let root = folder();
    write(root.path(), "d/a.txt", b"needle\n");
    for number in 0..50 {
        let name = format!("d/locked{number:02}.txt");
        write(root.path(), &name, b"needle\n");
        let path = root.path().join(&name);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).expect("lock");
    }
    let can_read = std::fs::read(root.path().join("d/locked00.txt")).is_ok();
    (root, !can_read)
}

fn unlock(root: &Path) {
    for number in 0..50 {
        let path = root.join(format!("d/locked{number:02}.txt"));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("unlock");
    }
}

#[test]
fn criterion_104_the_warnings_fit_after_the_results_and_are_not_results() {
    let (root, locked) = locked_folder();
    let server = FakeServer::constant(0.8);
    let run = ask(
        &server,
        root.path(),
        &["--json", "--max-chars", "1000"],
        &[],
    );
    unlock(root.path());
    if !locked {
        return;
    }
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000);
    assert!(document["warnings_left_out"].as_u64().unwrap_or(0) > 0);
    assert_eq!(document["size_limit_fix"], FIX_1000);
    // A warning is not a result, so there is no exit 8.
    assert_eq!(document["left_out"], 0);
    assert_eq!(run.code, Some(0));

    // The same for a model failure and for a search with no match.
    let (root, _) = locked_folder();
    let failing = FakeServer::start(|_| Reply::status(500));
    let run = ask(
        &failing,
        root.path(),
        &["--json", "--max-chars", "1000"],
        &[],
    );
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000);
    assert_eq!(run.code, Some(6));
    assert!(document["warnings_left_out"].as_u64().unwrap_or(0) > 0);
    let run = run_of(
        jag(folder().path(), &server.base_url())
            .current_dir(root.path())
            .args(["--json", "--max-chars", "1000", "--ask", "q", "absent", "d"])
            .output()
            .expect("run jag"),
    );
    unlock(root.path());
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000);
    assert_eq!(run.code, Some(1));
    assert!(document["warnings_left_out"].as_u64().unwrap_or(0) > 0);
}

// A path that is longer than the limit makes the error text shorter.
#[test]
fn an_error_document_with_a_long_key_file_path_fits_and_parses() {
    let root = folder();
    let mut home = root.path().to_path_buf();
    // The whole path must stay under 1,024 bytes, the limit of macOS.
    for part in 0..4 {
        home.push(format!("{part}{}", "h".repeat(215)));
    }
    let config = home.join(".config/jag");
    std::fs::create_dir_all(&config).expect("folders");
    let file = config.join("config.yml");
    std::fs::write(&file, "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\n").expect("write");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("mode");
    write(root.path(), "d/a.txt", b"needle\n");
    let run = run_of(
        jag_bare(&home)
            .current_dir(root.path())
            .args(["--json", "--max-chars", "1000", "--ask", "q", "needle", "d"])
            .output()
            .expect("run jag"),
    );
    // The key text is in the file only, and the document does not hold it.
    assert_eq!(run.code, Some(3));
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000, "{}", run.chars());
    assert!(
        document["error"]["cause"]
            .as_str()
            .unwrap_or("")
            .contains("...")
    );
    assert_eq!(document["size_limit_fix"], FIX_1000);
}

// DECISION-20: plain grep has the limit too.
#[test]
fn plain_grep_has_the_size_limit_and_makes_no_call() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = run(
        &server,
        root.path(),
        &["--max-chars", "1000", "needle"],
        &[],
    );
    assert_eq!(run.chars(), 904);
    assert_eq!(shown_names(&run.stdout), names(0, 7));
    assert!(!run.stdout.contains("--"));
    assert_eq!(run.stderr, notice(8, 30, "1000"));
    assert_eq!(run.code, Some(8));
    assert!(server.requests().is_empty());
}

#[test]
fn plain_grep_in_a_document_fits_the_limit() {
    let server = ranked_server();
    let root = ranked_folder();
    let run = run(
        &server,
        root.path(),
        &["--json", "--max-chars", "1000", "needle"],
        &[],
    );
    let document = parse_doc(&run);
    assert!(run.chars() <= 1000);
    let shown = document["results"].as_array().map_or(0, Vec::len);
    assert!(shown > 0 && shown < 30);
    assert_eq!(document["left_out"], 30 - shown);
    assert_eq!(document["exit_code"], 8);
    assert_eq!(run.code, Some(8));
    assert_eq!(run.stderr, "");
    assert!(server.requests().is_empty());
}

#[test]
fn plain_grep_reads_no_key_so_a_bad_variable_gives_exit_2() {
    let root = ranked_folder();
    let home = folder();
    let run = run_of(
        jag_bare(home.path())
            .current_dir(root.path())
            .env("JAG_MAX_CHARS", "abc")
            .args(["needle", "d"])
            .output()
            .expect("run jag"),
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert!(run.stderr.contains("JAG_MAX_CHARS is not a whole number"));
    // With a question, the key comes first.
    let run = run_of(
        jag_bare(home.path())
            .current_dir(root.path())
            .env("JAG_MAX_CHARS", "abc")
            .args(["--ask", "q", "needle", "d"])
            .output()
            .expect("run jag"),
    );
    assert_eq!(run.code, Some(3));
}
