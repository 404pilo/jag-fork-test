//! End-to-end tests of where jag searches (FEATURE-14): no path, the text of
//! a pipe, a mix of files and folders, and a path that does not exist
//! (CRITERION-35, 59, 60, 61, and 70, and decisions 3, 8, 9, 10, and 20).
//!
//! Each test with no path sets the current folder to a temporary folder, so
//! that the walk never reads the files of the crate. `assert_cmd` gives jag
//! an empty pipe that closes at once, or the piped text of `write_stdin`.
//! `jag_std` with `Stdio::null()` gives jag no pipe.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Output, Stdio};

use common::fake_server::FakeServer;
use common::{KEY, jag, jag_bare, jag_std, write};
use jag::output::NO_GOOD_ANSWER;
use serde_json::{Value, json};
use tempfile::TempDir;

const QUESTION: &str = "which line has the needle?";
const DASH_FAILURE: &str = "jag: the path - is not valid.\nTo search the text of a pipe, give no path. To search a file with the name -, give ./-.\n";
const EMPTY_FAILURE: &str =
    "jag: a path is empty.\nGive the path of a file or a folder, or give no path.\n";

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Run {
    fn document(&self) -> Value {
        assert_eq!(self.stdout.lines().count(), 1, "{}", self.stdout);
        serde_json::from_str(&self.stdout).expect("one JSON document")
    }
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

/// Run `jag ARGS` in `cwd` with the key and the fake server. `stdin` is the
/// piped text. `None` gives the empty pipe of `assert_cmd`.
fn run(server: &FakeServer, cwd: &Path, args: &[&str], stdin: Option<&[u8]>) -> Run {
    let home = folder();
    let mut command = jag(home.path(), &server.base_url());
    command.current_dir(cwd).args(args);
    if let Some(bytes) = stdin {
        command.write_stdin(bytes.to_vec());
    }
    run_of(command.output().expect("run jag"))
}

/// Run `jag ARGS` in `cwd` with no pipe: standard input is `/dev/null`.
fn run_no_pipe(server: &FakeServer, cwd: &Path, args: &[&str]) -> Run {
    let home = folder();
    run_of(
        jag_std(home.path(), &server.base_url())
            .current_dir(cwd)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .expect("run jag"),
    )
}

/// Run `jag ARGS` in `cwd` with no key and no setting.
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

/// A folder with `a.txt` and `sub/b.txt`, each with `needle` on line 1.
fn two_files() -> TempDir {
    let root = folder();
    write(root.path(), "a.txt", b"needle\n");
    write(root.path(), "sub/b.txt", b"needle\n");
    root
}

/// A folder with `a.txt` that holds `needle` on line 1.
fn one_file() -> TempDir {
    let root = folder();
    write(root.path(), "a.txt", b"needle\n");
    root
}

// CRITERION-59: no path and no pipe give the current folder, with no `./`.
#[test]
fn criterion_59_no_path_and_no_pipe_search_the_current_folder() {
    let root = two_files();
    let server = FakeServer::constant(0.8);
    let plain = run_no_pipe(&server, root.path(), &["needle"]);
    assert_eq!(plain.code, Some(0), "{}", plain.stderr);
    assert_eq!(plain.stdout, "a.txt:1:needle\nsub/b.txt:1:needle\n");
    assert!(server.requests().is_empty());

    let asked = run_no_pipe(&server, root.path(), &["--ask", QUESTION, "needle"]);
    assert_eq!(asked.code, Some(0), "{}", asked.stderr);
    assert!(asked.stdout.contains("a.txt:1:needle"), "{}", asked.stdout);
    assert!(
        asked.stdout.contains("sub/b.txt:1:needle"),
        "{}",
        asked.stdout
    );
    assert!(!asked.stdout.lines().any(|line| line.starts_with("./")));
    assert_eq!(server.requests().len(), 1);
}

// CRITERION-59 and decision 5: no path gives the search of `.`, with no `./`.
#[test]
fn criterion_59_no_path_is_the_search_of_dot_with_no_dot_slash() {
    let root = folder();
    write(root.path(), "a.txt", b"needle\n");
    write(root.path(), "sub/c.txt", b"x needle\n");
    write(root.path(), ".hidden.txt", b"needle\n");
    write(root.path(), ".ignore", b"ignored.txt\n");
    write(root.path(), "ignored.txt", b"needle\n");
    write(root.path(), "binary.dat", b"needle\0\n");
    let server = FakeServer::constant(0.8);
    let no_path = run_no_pipe(&server, root.path(), &["needle"]);
    let dot = run_no_pipe(&server, root.path(), &["needle", "."]);
    assert_eq!(no_path.code, Some(0));
    assert_eq!(no_path.code, dot.code);
    assert_eq!(no_path.stderr, dot.stderr);
    assert!(
        no_path.stderr.contains("jag: skipped"),
        "{}",
        no_path.stderr
    );
    assert_eq!(no_path.stdout, dot.stdout.replace("./", ""));
    assert_eq!(no_path.stdout, "a.txt:1:needle\nsub/c.txt:1:x needle\n");
}

// Decision 3: an empty pipe gives the empty text, as rg does.
#[test]
fn decision_3_an_empty_pipe_gives_exit_1_and_no_call() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let plain = run(&server, root.path(), &["needle"], None);
    assert_eq!(plain.code, Some(1));
    assert_eq!(plain.stdout, "");
    assert_eq!(plain.stderr, "");

    let asked = run(&server, root.path(), &["--ask", QUESTION, "needle"], None);
    assert_eq!(asked.code, Some(1));
    assert_eq!(asked.stdout, "");
    assert!(server.requests().is_empty());

    let json = run(&server, root.path(), &["--json", "needle"], None);
    assert_eq!(json.code, Some(1));
    let document = json.document();
    assert_eq!(document["results"], json!([]));
    assert_eq!(document["exit_code"], 1);
    assert_eq!(document["error"], Value::Null);
    assert!(server.requests().is_empty());
}

// CRITERION-60: no path and piped text give the piped text.
#[test]
fn criterion_60_no_path_and_piped_text_search_the_piped_text() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let text: &[u8] = b"x\nneedle here\n";
    let plain = run(&server, root.path(), &["needle"], Some(text));
    assert_eq!(plain.code, Some(0), "{}", plain.stderr);
    assert_eq!(plain.stdout, "(standard input):2:needle here\n");

    let asked = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle"],
        Some(text),
    );
    assert_eq!(asked.code, Some(0), "{}", asked.stderr);
    assert_eq!(
        asked.stdout,
        "(standard input)-1-x\n(standard input):2:needle here\n"
    );
}

// Decision 4: a path wins over the pipe.
#[test]
fn decision_4_a_path_wins_over_piped_text() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["needle", "a.txt"],
        Some(b"x\nneedle here\n"),
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:1:needle\n");
}

const SEVEN_LINES: &[u8] = b"a\nb\nc\nd\ne\nf\nneedle\n";

// CRITERION-35: the label and the line numbers of the piped text.
#[test]
fn criterion_35_a_result_of_piped_text_has_the_label_and_its_line_numbers() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let asked = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle"],
        Some(SEVEN_LINES),
    );
    assert_eq!(asked.code, Some(0), "{}", asked.stderr);
    assert_eq!(
        asked.stdout,
        "(standard input)-4-d\n(standard input)-5-e\n(standard input)-6-f\n(standard input):7:needle\n"
    );
    let plain = run(&server, root.path(), &["needle"], Some(SEVEN_LINES));
    assert_eq!(plain.stdout, "(standard input):7:needle\n");
}

// CRITERION-35 and decision 16: in structured mode, the path is null.
#[test]
fn criterion_35_a_result_of_piped_text_has_a_null_path_in_structured_mode() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["--json", "--ask", QUESTION, "needle"],
        Some(SEVEN_LINES),
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    let document = run.document();
    let result = &document["results"][0];
    assert_eq!(result["path"], Value::Null);
    assert_eq!(result["first_line"], 4);
    assert_eq!(result["last_line"], 7);
    let marks: Vec<(u64, bool)> = result["lines"]
        .as_array()
        .expect("lines")
        .iter()
        .map(|line| {
            (
                line["number"].as_u64().expect("number"),
                line["is_match"].as_bool().expect("is_match"),
            )
        })
        .collect();
    assert_eq!(marks, [(4, false), (5, false), (6, false), (7, true)]);
}

// CRITERION-35 and decision 11: binary piped text gives the binary rule.
#[test]
fn criterion_35_binary_piped_text_gives_no_result_and_the_skip_notice() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle"],
        Some(b"needle\0\n"),
    );
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "jag: skipped 1 files and folders (1 binary, 0 hidden, 0 ignored)\n"
    );
    assert!(server.requests().is_empty());
}

/// The folder of CRITERION-61.
fn mixed() -> TempDir {
    let root = folder();
    write(root.path(), "one.txt", b"needle\n");
    write(root.path(), "dir/two.txt", b"needle\n");
    write(root.path(), "dir/sub/three.txt", b"needle\n");
    write(root.path(), "other/four.txt", b"needle\n");
    root
}

// CRITERION-61: several paths, each a file or a folder.
#[test]
fn criterion_61_several_paths_each_a_file_or_a_folder() {
    let root = mixed();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["needle", "one.txt", "dir", "other/four.txt"],
        None,
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "one.txt:1:needle\ndir/sub/three.txt:1:needle\ndir/two.txt:1:needle\nother/four.txt:1:needle\n"
    );
}

// CRITERION-61 and decision 6: a hidden file that the user names is searched.
#[test]
fn criterion_61_a_hidden_file_as_a_path_gives_its_result() {
    let root = one_file();
    write(root.path(), ".hidden.txt", b"needle\n");
    let server = FakeServer::constant(0.8);
    let named = run(&server, root.path(), &["needle", ".hidden.txt"], None);
    assert_eq!(named.code, Some(0));
    assert_eq!(named.stdout, ".hidden.txt:1:needle\n");
    assert_eq!(named.stderr, "");

    let walked = run(&server, root.path(), &["needle", "."], None);
    assert_eq!(walked.stdout, "./a.txt:1:needle\n");
    assert_eq!(
        walked.stderr,
        "jag: skipped 1 files and folders (0 binary, 1 hidden, 0 ignored)\n"
    );
}

// Decision 6: a file that the user names two times gives two results.
#[test]
fn decision_6_a_file_named_two_times_gives_two_results() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle", "a.txt", "a.txt"],
        None,
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "a.txt:1:needle\n--\na.txt:1:needle\n");
}

// CRITERION-70: a path that does not exist.
#[test]
fn criterion_70_a_path_that_does_not_exist_gives_exit_2_and_no_call() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    for args in [
        &["needle", "nofile"][..],
        &["--ask", QUESTION, "needle", "nofile"],
    ] {
        let run = run(&server, root.path(), args, None);
        assert_eq!(run.code, Some(2), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
        assert_eq!(
            run.stderr, "jag: the path nofile does not exist\n",
            "{args:?}"
        );
    }
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_70_jag_searches_the_other_paths_and_exits_2() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let plain = run(&server, root.path(), &["needle", "a.txt", "nofile"], None);
    assert_eq!(plain.code, Some(2));
    assert_eq!(plain.stdout, "a.txt:1:needle\n");
    assert_eq!(plain.stderr, "jag: the path nofile does not exist\n");
    assert!(server.requests().is_empty());

    let asked = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle", "a.txt", "nofile"],
        None,
    );
    assert_eq!(asked.code, Some(2));
    assert_eq!(asked.stdout, "a.txt:1:needle\n");
    assert!(
        asked
            .stderr
            .contains("jag: the path nofile does not exist\n"),
        "{}",
        asked.stderr
    );
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn criterion_70_the_warnings_come_in_the_order_of_the_command_line() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["needle", "nofile2", "a.txt", "nofile1"],
        None,
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        "jag: the path nofile2 does not exist\njag: the path nofile1 does not exist\n"
    );
}

#[test]
fn criterion_70_a_broken_link_and_a_path_under_a_file_do_not_exist() {
    let root = one_file();
    std::os::unix::fs::symlink(root.path().join("gone.txt"), root.path().join("link.txt"))
        .expect("make link");
    let server = FakeServer::constant(0.8);
    for path in ["link.txt", "a.txt/x"] {
        let run = run(&server, root.path(), &["needle", path], None);
        assert_eq!(run.code, Some(2), "{path}");
        assert_eq!(
            run.stderr,
            format!("jag: the path {path} does not exist\n"),
            "{path}"
        );
    }
}

// CRITERION-70 and decision 16: the document of a path that does not exist.
#[test]
fn criterion_70_the_document_names_the_path_in_its_warnings() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run_with = |args: &[&str]| run(&server, root.path(), args, None);

    let both = run_with(&["--json", "needle", "a.txt", "nofile"]);
    assert_eq!(both.code, Some(2));
    assert_eq!(both.stderr, "");
    let document = both.document();
    assert_eq!(document["exit_code"], 2);
    assert_eq!(document["error"], Value::Null);
    assert_eq!(
        document["warnings"],
        json!([{"path": "nofile", "text": "the path nofile does not exist"}])
    );
    assert_eq!(document["results"][0]["path"], "a.txt");
    assert_eq!(document["results"].as_array().map(Vec::len), Some(1));

    let alone = run_with(&["--json", "needle", "nofile"]);
    assert_eq!(alone.code, Some(2));
    let document = alone.document();
    assert_eq!(document["results"], json!([]));
    assert_eq!(document["exit_code"], 2);
    assert_eq!(document["error"], Value::Null);
}

// Decision 20: 2 wins over 1 and 7, and the label and the notice still show.
#[test]
fn decision_20_exit_2_wins_and_the_label_of_no_good_answer_still_shows() {
    let root = one_file();
    let server = FakeServer::constant(0.2);
    let run = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle", "a.txt", "nofile"],
        None,
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout.lines().next(), Some(NO_GOOD_ANSWER));
}

#[test]
fn decision_20_exit_2_wins_and_the_cost_limit_notice_still_shows() {
    let root = one_file();
    write(root.path(), "b.txt", b"needle\n");
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &[
            "--ask",
            QUESTION,
            "--max-scored",
            "1",
            "needle",
            "a.txt",
            "b.txt",
            "nofile",
        ],
        None,
    );
    assert_eq!(run.code, Some(2));
    assert!(
        run.stderr
            .contains("jag: cost limit reached: scored 1 of 2 results, skipped 1."),
        "{}",
        run.stderr
    );
}

// Decision 9: the key and the term come before the scan.
#[test]
fn decision_9_a_missing_key_and_a_bad_term_come_before_the_warning() {
    let root = one_file();
    let no_key = run_bare(root.path(), &["--ask", QUESTION, "needle", "nofile"]);
    assert_eq!(no_key.code, Some(3));
    assert!(
        !no_key.stderr.contains("does not exist"),
        "{}",
        no_key.stderr
    );

    let bad_term = run_bare(root.path(), &["(", "nofile"]);
    assert_eq!(bad_term.code, Some(2));
    assert!(
        bad_term
            .stderr
            .starts_with("jag: the term is not a valid regular expression."),
        "{}",
        bad_term.stderr
    );
    assert!(!bad_term.stderr.contains("does not exist"));
}

// Decision 8: the path `-` and the empty path are usage errors.
#[test]
fn decision_8_the_path_dash_is_a_usage_error() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let with_key = run(&server, root.path(), &["needle", "-"], Some(b"needle\n"));
    assert_eq!(with_key.code, Some(2));
    assert_eq!(with_key.stdout, "");
    assert_eq!(with_key.stderr, DASH_FAILURE);

    let no_key = run_bare(root.path(), &["--ask", QUESTION, "needle", "-"]);
    assert_eq!(no_key.code, Some(2));
    assert_eq!(no_key.stderr, DASH_FAILURE);

    write(root.path(), "-", b"needle in dash\n");
    let file = run(&server, root.path(), &["needle", "a.txt", "-"], None);
    assert_eq!(file.code, Some(2));
    assert_eq!(file.stderr, DASH_FAILURE);
    let dot_slash = run(&server, root.path(), &["needle", "./-"], None);
    assert_eq!(dot_slash.code, Some(0));
    assert_eq!(dot_slash.stdout, "./-:1:needle in dash\n");
    assert!(server.requests().is_empty());
}

#[test]
fn decision_8_the_empty_path_is_a_usage_error() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    for args in [&["needle", ""][..], &["needle", "a.txt", ""]] {
        let run = run(&server, root.path(), args, None);
        assert_eq!(run.code, Some(2), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
        assert_eq!(run.stderr, EMPTY_FAILURE, "{args:?}");
        assert!(!run.stderr.contains("  "));
    }
}

// Decision 10: a path that exists and that jag cannot read is a read error.
#[test]
fn decision_10_a_locked_file_or_folder_is_a_read_error_with_exit_1() {
    let root = one_file();
    write(root.path(), "locked.txt", b"needle\n");
    write(root.path(), "closed/a.txt", b"needle\n");
    let locked = root.path().join("locked.txt");
    let closed = root.path().join("closed");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).expect("lock");
    std::fs::set_permissions(&closed, std::fs::Permissions::from_mode(0o000)).expect("lock");
    // A user who reads any file (root) cannot run this test.
    let can_read = std::fs::read(&locked).is_ok();
    let server = FakeServer::constant(0.8);
    let file = run(
        &server,
        root.path(),
        &["--ask", QUESTION, "needle", "locked.txt"],
        None,
    );
    let folder = run(&server, root.path(), &["needle", "closed"], None);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644)).expect("unlock");
    std::fs::set_permissions(&closed, std::fs::Permissions::from_mode(0o755)).expect("unlock");
    if can_read {
        return;
    }
    assert_eq!(file.code, Some(1));
    assert!(
        file.stderr.contains("jag: cannot read locked.txt"),
        "{}",
        file.stderr
    );
    assert!(server.requests().is_empty());
    assert_eq!(folder.code, Some(1));
    assert!(
        folder.stderr.contains("jag: cannot read closed"),
        "{}",
        folder.stderr
    );
    assert!(!folder.stderr.contains("does not exist"));
}

// FEATURE-13 with FEATURE-14: `-q`, `-l`, and `-c` with a path that does not
// exist, and with piped text.
#[test]
fn feature_13_q_with_a_match_exits_0_even_with_a_missing_path() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["-q", "needle", "a.txt", "nofile"],
        None,
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "jag: the path nofile does not exist\n");
    assert!(server.requests().is_empty());
}

#[test]
fn feature_13_the_globs_do_not_apply_to_piped_text() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["--exclude=*", "needle"],
        Some(SEVEN_LINES),
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "(standard input):7:needle\n");
    assert!(server.requests().is_empty());
}

#[test]
fn feature_13_q_with_no_match_and_a_missing_path_exits_2() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let run = run(
        &server,
        root.path(),
        &["-q", "absent", "a.txt", "nofile"],
        None,
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, "jag: the path nofile does not exist\n");
}

#[test]
fn feature_13_l_and_c_write_the_lines_of_the_other_paths_and_exit_2() {
    let root = one_file();
    let server = FakeServer::constant(0.8);
    let files = run(
        &server,
        root.path(),
        &["-l", "needle", "a.txt", "nofile"],
        None,
    );
    assert_eq!(files.code, Some(2));
    assert_eq!(files.stdout, "a.txt\n");
    assert_eq!(files.stderr, "jag: the path nofile does not exist\n");
    write(root.path(), "b.txt", b"other\n");
    let counts = run(
        &server,
        root.path(),
        &["-c", "needle", "nofile", "a.txt", "b.txt"],
        None,
    );
    assert_eq!(counts.code, Some(2));
    assert_eq!(counts.stdout, "a.txt:1\nb.txt:0\n");
    assert_eq!(counts.stderr, "jag: the path nofile does not exist\n");
}

#[test]
fn feature_13_l_and_c_label_piped_text() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let files = run(&server, root.path(), &["-l", "needle"], Some(SEVEN_LINES));
    assert_eq!(files.code, Some(0));
    assert_eq!(files.stdout, "(standard input)\n");
    let counts = run(&server, root.path(), &["-c", "needle"], Some(SEVEN_LINES));
    assert_eq!(counts.code, Some(0));
    assert_eq!(counts.stdout, "(standard input):1\n");
}

#[test]
fn feature_13_c_with_an_empty_pipe_writes_a_zero_count_and_exits_1() {
    let root = folder();
    let server = FakeServer::constant(0.8);
    let run = run(&server, root.path(), &["-c", "needle"], Some(b""));
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "(standard input):0\n");
    assert!(server.requests().is_empty());
}
