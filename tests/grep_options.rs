//! End-to-end tests of the common grep options and the exit codes
//! (FEATURE-13, CRITERION-55 to CRITERION-58 and CRITERION-69).
//!
//! The expected bytes come from GNU grep 3.12, checked on 2026-10-14. No test
//! runs the `grep` program. A test that proves "no model call" uses the fake
//! server and checks that it has no request.
//!
//! The cases of the spec that need FEATURE-14 (piped text, an empty pipe, and
//! a path that does not exist) are not here: that code is not on `main`.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;

use common::fake_server::FakeServer;
use common::{KEY, jag, jag_bare, write};
use serde_json::Value;
use tempfile::TempDir;

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

/// `a.txt` has two matching lines, and `b.txt` has none.
fn two_files() -> TempDir {
    let root = folder();
    write(root.path(), "a.txt", b"one needle\ntwo\nthree needle\n");
    write(root.path(), "b.txt", b"x\n");
    root
}

/// `ctx.txt` has the lines `1` to `11`, with `X` at lines 4 and 10.
fn context_file() -> TempDir {
    let root = folder();
    let text: String = (1..=11)
        .map(|number| {
            if number == 4 || number == 10 {
                "X\n".to_owned()
            } else {
                format!("{number}\n")
            }
        })
        .collect();
    write(root.path(), "ctx.txt", text.as_bytes());
    root
}

/// The lines that the model got for the first request, one text for each
/// result.
fn texts(server: &FakeServer) -> Vec<String> {
    server.requests()[0].snippet_texts()
}

// CRITERION-55: the options, with their grep meaning.

#[test]
fn criterion_55_i_matches_upper_and_lower_case_letters_as_the_same() {
    let root = folder();
    write(root.path(), "a.txt", b"a Needle here\nnothing\n");
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["-i", "--ask", "q", "needle", "a.txt"],
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(texts(&server), ["a Needle here\nnothing"]);
    let run = self::run(&server, root.path(), &["--ask", "q", "needle", "a.txt"]);
    assert_eq!(run.code, Some(1));
}

#[test]
fn criterion_55_w_matches_whole_words_only() {
    let root = folder();
    write(
        root.path(),
        "a.txt",
        b"the needle.\nneedles\nneedle_x\nNeedle\n",
    );
    let run = run_bare(root.path(), &["-w", "needle", "a.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:1:the needle.\n");
    let long = run_bare(root.path(), &["--word-regexp", "needle", "a.txt"]);
    assert_eq!(long.stdout, run.stdout);
}

#[test]
fn criterion_55_e_gives_more_than_one_term_and_each_word_is_a_path() {
    let root = folder();
    write(root.path(), "a.txt", b"one\ntwo\nthree\n");
    write(root.path(), "b.txt", b"nothing\nthree again\n");
    let run = run_bare(root.path(), &["-e", "one", "-e", "three", "a.txt", "b.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(
        run.stdout,
        "a.txt:1:one\na.txt:3:three\nb.txt:2:three again\n"
    );
    // A term that starts with a dash, and the long name.
    write(root.path(), "c.txt", b"a -x b\n");
    let run = run_bare(root.path(), &["-e", "-x", "c.txt"]);
    assert_eq!(run.stdout, "c.txt:1:a -x b\n");
    let run = run_bare(root.path(), &["--regexp=-x", "c.txt"]);
    assert_eq!(run.stdout, "c.txt:1:a -x b\n");
}

#[test]
fn criterion_55_with_e_the_question_comes_only_from_ask_and_ask_can_be_a_term() {
    let root = folder();
    write(root.path(), "a.txt", b"needle\nuse --ask here\n");
    let server = FakeServer::constant(0.9);
    // `-e --ask q needle` searches the term `--ask` in the paths `q` and
    // `needle`: both are missing, so jag finds nothing and makes no call.
    let run = run(&server, root.path(), &["-e", "--ask", "q", "needle"]);
    assert_eq!(run.stdout, "");
    assert!(server.requests().is_empty());
    let run = self::run(&server, root.path(), &["-e", "--ask", "a.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:2:use --ask here\n");
    assert!(server.requests().is_empty());
    let run = self::run(
        &server,
        root.path(),
        &["--ask", "which?", "-e", "needle", "a.txt"],
    );
    assert_eq!(run.code, Some(0));
    assert_eq!(server.requests().len(), 1);
}

#[test]
fn criterion_56_f_reads_the_term_as_fixed_text() {
    let root = folder();
    write(root.path(), "file", b"x a( y\nneedles\nthe needle.\n");
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["-F", "--ask", "q", "a(", "file"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(texts(&server), ["x a( y\nneedles\nthe needle."]);
    let run = self::run(&server, root.path(), &["--ask", "q", "a(", "file"]);
    assert_eq!(run.code, Some(2));
    assert!(run.stderr.contains("not a valid regular expression"));
    assert_eq!(server.requests().len(), 1);
    // With `-w`, `-F` still applies, and `.` is not any character.
    let run = run_bare(root.path(), &["-w", "-F", "needle.", "file"]);
    assert_eq!(run.stdout, "file:3:the needle.\n");
    let run = run_bare(root.path(), &["-F", "needle.", "file"]);
    assert_eq!(run.stdout, "file:3:the needle.\n");
}

#[test]
fn criterion_55_include_and_exclude_follow_the_rule_of_gnu_grep() {
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle a\n");
    write(root.path(), "dir/c.rs", b"needle c\n");
    write(root.path(), "dir/d.md", b"needle d\n");
    write(root.path(), "dir/sub/e.rs", b"needle e\n");
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["--include=*.rs", "--ask", "q", "needle", "dir"],
    );
    assert_eq!(run.code, Some(0));
    let mut got = texts(&server);
    got.sort();
    assert_eq!(got, ["needle c", "needle e"]);
    // The last glob that matches decides.
    let run = run_bare(
        root.path(),
        &["--exclude=*.md", "--include=*.rs", "needle", "dir"],
    );
    assert_eq!(
        run.stdout,
        "dir/a.txt:1:needle a\ndir/c.rs:1:needle c\ndir/sub/e.rs:1:needle e\n"
    );
    let run = run_bare(
        root.path(),
        &["--include=*.rs", "--exclude=c.*", "needle", "dir"],
    );
    assert_eq!(run.stdout, "dir/sub/e.rs:1:needle e\n");
    // A glob applies to a file that the user names.
    let run = run_bare(root.path(), &["--include=*.rs", "needle", "dir/a.txt"]);
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    // A glob does not skip a folder.
    let run = run_bare(root.path(), &["--exclude=sub", "needle", "dir"]);
    assert!(run.stdout.contains("dir/sub/e.rs:1:needle e\n"));
    // Braces: a rule of ripgrep.
    let run = run_bare(root.path(), &["--include=*.{md,txt}", "needle", "dir"]);
    assert_eq!(run.stdout, "dir/a.txt:1:needle a\ndir/d.md:1:needle d\n");
}

#[test]
fn criterion_55_a_glob_that_skips_the_only_file_gives_exit_1_and_no_call() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["--ask", "q", "--exclude=*.txt", "needle", "a.txt"],
    );
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "");
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_55_a_file_that_a_glob_skips_is_not_in_the_skip_notice() {
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle\n");
    write(root.path(), "dir/b.md", b"needle\n");
    let run = run_bare(root.path(), &["--exclude=*.md", "needle", "dir"]);
    assert_eq!(run.stdout, "dir/a.txt:1:needle\n");
    assert_eq!(run.stderr, "");
}

#[test]
fn criterion_55_a_glob_that_does_not_parse_exits_2_and_is_not_shown() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    for (option, name) in [("--include", "--include"), ("--exclude", "--exclude")] {
        let glob = format!("{option}=SENTINEL[");
        let run = run(
            &server,
            root.path(),
            &["--ask", "q", &glob, "needle", "a.txt"],
        );
        assert_eq!(run.code, Some(2));
        assert_eq!(run.stdout, "");
        assert_eq!(
            run.stderr,
            format!(
                "jag: the glob of {name} is not valid.\nCheck the glob. jag uses the glob rules of ripgrep.\n"
            )
        );
    }
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_55_the_options_conflict_with_check_setup_and_forget_key() {
    let root = two_files();
    for mode in ["--check", "--setup", "--forget-key"] {
        for option in ["-i", "-w", "-F", "-l", "-c", "-q", "--include=*"] {
            let run = run_bare(root.path(), &[mode, option]);
            assert_eq!(run.code, Some(2), "{mode} {option}");
            assert_eq!(
                run.stderr,
                "jag: the command line is not valid. Run jag --help to see the usage.\n",
                "{mode} {option}"
            );
        }
    }
}

// FEATURE-13 decision 37: a short option reads `=` as the separator, as rg
// does. GNU grep reads it as a part of the value.

#[test]
fn decision_37_e_equals_gives_the_term_after_the_equals_sign() {
    let root = folder();
    write(root.path(), "a.txt", b"foo\n=foo\nbar\n");
    let run = run_bare(root.path(), &["-e=foo", "a.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:1:foo\na.txt:2:=foo\n");
    let run = run_bare(root.path(), &["-e", "=foo", "a.txt"]);
    assert_eq!(run.stdout, "a.txt:2:=foo\n");
    let run = run_bare(root.path(), &["-e=", "a.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:1:foo\na.txt:2:=foo\na.txt:3:bar\n");
    let run = run_bare(root.path(), &["-ie=FOO", "a.txt"]);
    assert_eq!(run.stdout, "a.txt:1:foo\na.txt:2:=foo\n");
}

#[test]
fn decision_37_a_equals_gives_the_number_after_the_equals_sign() {
    let root = folder();
    write(root.path(), "f.txt", b"1\n2\n3\nneedle\n5\n6\n7\n8\n");
    let run = run_bare(root.path(), &["-A=2", "needle", "f.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "f.txt:4:needle\nf.txt-5-5\nf.txt-6-6\n");
}

// FEATURE-13 decision 38: an argument error writes the error document only
// when clap reads `--json` as the option.

#[test]
fn decision_38_json_as_the_value_of_e_gives_the_text_error() {
    let root = two_files();
    let run = run_bare(root.path(), &["-e", "--json", "--nope", "a.txt"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "jag: the option --nope is not valid.\nRun jag --help to see the options.\n"
    );
}

#[test]
fn decision_38_a_double_dash_as_the_value_of_e_does_not_end_the_options() {
    let root = two_files();
    let run = run_bare(root.path(), &["-e", "--", "--json", "--nope", "a.txt"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stderr, "");
    let document: Value = serde_json::from_str(&run.stdout).expect("a document");
    assert_eq!(document["exit_code"], 2);
    assert_eq!(
        document["error"]["cause"],
        "the option --nope is not valid."
    );
}

// CRITERION-57: an unknown option.

#[test]
fn criterion_57_an_unknown_option_gives_a_usage_error_that_names_the_option() {
    let root = two_files();
    for (args, name) in [
        (&["--nope", "needle", "a.txt"][..], "--nope"),
        (&["--nope=SENTINEL", "needle", "a.txt"], "--nope"),
        (&["-iZx", "needle", "a.txt"], "-Z"),
        (&["-r", "needle", "a.txt"], "-r"),
        (&["-rn", "needle", "."], "-r"),
    ] {
        let run = run_bare(root.path(), args);
        assert_eq!(run.code, Some(2), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
        assert_eq!(
            run.stderr,
            format!("jag: the option {name} is not valid.\nRun jag --help to see the options.\n"),
            "{args:?}"
        );
        assert!(!run.stderr.contains("SENTINEL"));
    }
}

#[test]
fn criterion_57_a_long_text_that_could_be_a_key_gives_the_fixed_line() {
    let root = two_files();
    let run = run_bare(
        root.path(),
        &["--SENTINEL-SENTINEL-SENTINEL-SENTINEL", "needle", "a.txt"],
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(
        run.stderr,
        "jag: the command line is not valid. Run jag --help to see the usage.\n"
    );
}

#[test]
fn criterion_57_the_options_of_jag_are_not_unknown_options() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    for args in [
        &["--color=always", "needle", "a.txt"][..],
        &["--colour", "needle", "a.txt"],
        &["--json", "needle", "a.txt"],
        &["--ask", "q", "--timeout", "5", "needle", "a.txt"],
        &["--ask", "q", "--max-scored", "10", "needle", "a.txt"],
        &["--max-chars", "20000", "needle", "a.txt"],
        &["--no-max-chars", "needle", "a.txt"],
        &["--ask", "q", "--max-chars", "20000", "needle", "a.txt"],
        &["--ask", "q", "--no-max-chars", "needle", "a.txt"],
    ] {
        let run = run(&server, root.path(), args);
        assert_eq!(run.code, Some(0), "{args:?}: {}", run.stderr);
    }
}

// CRITERION-69 and CRITERION-55: the lines around a match.

#[test]
fn criterion_69_with_ask_and_no_option_a_result_has_3_lines_on_each_side() {
    let root = folder();
    write(root.path(), "f.txt", b"1\n2\n3\nneedle\n5\n6\n7\n8\n9\n");
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--ask", "q", "needle", "f.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(texts(&server), ["1\n2\n3\nneedle\n5\n6\n7"]);
}

#[test]
fn criterion_69_with_ask_the_options_set_the_lines_around_a_match() {
    let root = folder();
    let text: String = (1..=20)
        .map(|number| {
            if number == 4 {
                "needle\n".to_owned()
            } else {
                format!("{number}\n")
            }
        })
        .collect();
    write(root.path(), "f.txt", text.as_bytes());
    let cases: [(&[&str], &str); 3] = [
        (&["-C", "5"], "1\n2\n3\nneedle\n5\n6\n7\n8\n9"),
        (&["-A", "2"], "needle\n5\n6"),
        (&["-B", "1", "-A", "0"], "3\nneedle"),
    ];
    for (options, want) in cases {
        let server = FakeServer::constant(0.9);
        let mut args = options.to_vec();
        args.extend_from_slice(&["--ask", "q", "needle", "f.txt"]);
        let run = run(&server, root.path(), &args);
        assert_eq!(run.code, Some(0), "{options:?}");
        assert_eq!(texts(&server), [want], "{options:?}");
    }
}

#[test]
fn criterion_69_with_ask_more_than_12_lines_gives_exit_2_and_no_call() {
    let root = context_file();
    let server = FakeServer::constant(0.9);
    let run = run(
        &server,
        root.path(),
        &["-C", "13", "--ask", "q", "X", "ctx.txt"],
    );
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "jag: the value of -C is more than 12 with --ask.\nGive a number from 0 to 12.\n"
    );
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_69_a_value_that_is_not_a_whole_number_gives_exit_2_and_is_not_shown() {
    let root = context_file();
    for value in ["abc", "-1", "+1", "", "SENTINEL"] {
        let run = run_bare(root.path(), &["-A", value, "X", "ctx.txt"]);
        assert_eq!(run.code, Some(2), "{value:?}");
        assert_eq!(
            run.stderr,
            "jag: the value of -A is not a whole number of 0 or more.\nGive a number of lines, for example 3.\n",
            "{value:?}"
        );
    }
}

#[test]
fn criterion_69_plain_grep_writes_groups_with_a_double_dash_as_grep_does() {
    let root = context_file();
    let run = run_bare(root.path(), &["-C", "1", "X", "ctx.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(
        run.stdout,
        "ctx.txt-3-3\nctx.txt:4:X\nctx.txt-5-5\n--\nctx.txt-9-9\nctx.txt:10:X\nctx.txt-11-11\n"
    );
    let run = run_bare(root.path(), &["-C", "0", "X", "ctx.txt"]);
    assert_eq!(run.stdout, "ctx.txt:4:X\n--\nctx.txt:10:X\n");
    let run = run_bare(root.path(), &["-A", "1", "X", "ctx.txt"]);
    assert_eq!(
        run.stdout,
        "ctx.txt:4:X\nctx.txt-5-5\n--\nctx.txt:10:X\nctx.txt-11-11\n"
    );
    // Groups that touch join, with no limit of lines.
    let run = run_bare(root.path(), &["-C", "100", "X", "ctx.txt"]);
    assert_eq!(run.stdout.lines().count(), 11);
    assert!(!run.stdout.contains("--"));
    // With none of the options, one line for each match, and no `--`.
    let run = run_bare(root.path(), &["X", "ctx.txt"]);
    assert_eq!(run.stdout, "ctx.txt:4:X\nctx.txt:10:X\n");
}

#[test]
fn criterion_69_groups_of_two_files_are_set_apart_by_a_double_dash() {
    let root = folder();
    write(root.path(), "a.txt", b"x\nX\ny\n");
    write(root.path(), "b.txt", b"X\n");
    let run = run_bare(root.path(), &["-C", "1", "X", "a.txt", "b.txt"]);
    assert_eq!(
        run.stdout,
        "a.txt-1-x\na.txt:2:X\na.txt-3-y\n--\nb.txt:1:X\n"
    );
}

#[test]
fn criterion_69_plain_grep_with_json_gives_one_result_for_each_group() {
    let root = context_file();
    let run = run_bare(root.path(), &["--json", "-C", "1", "X", "ctx.txt"]);
    assert_eq!(run.code, Some(0));
    let document: Value = serde_json::from_str(&run.stdout).expect("one document");
    assert_eq!(document["results_found"], 2);
    let results = document["results"].as_array().expect("a list");
    assert_eq!(results.len(), 2);
    for result in results {
        assert_eq!(result["lines"].as_array().map(Vec::len), Some(3));
        assert_eq!(result["lines"][1]["is_match"], true);
        assert_eq!(result["lines"][0]["is_match"], false);
    }
}

#[test]
fn criterion_69_color_marks_the_double_dash_and_the_match() {
    let root = context_file();
    let run = run_bare(root.path(), &["--color=always", "-C", "0", "X", "ctx.txt"]);
    assert!(
        run.stdout.contains("\n\x1b[36m\x1b[K--\x1b[m\x1b[K\n"),
        "{:?}",
        run.stdout
    );
    assert!(run.stdout.contains("\x1b[01;31m\x1b[KX\x1b[m\x1b[K"));
}

// CRITERION-55: -l, -c, and -q.

#[test]
fn criterion_55_l_writes_the_path_of_each_file_with_a_match() {
    let root = two_files();
    let run = run_bare(root.path(), &["-l", "needle", "a.txt", "b.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt\n");
    let long = run_bare(
        root.path(),
        &["--files-with-matches", "needle", "a.txt", "b.txt"],
    );
    assert_eq!(long.stdout, "a.txt\n");
    // A file that the user names two times shows two times.
    let run = run_bare(root.path(), &["-l", "needle", "a.txt", "a.txt"]);
    assert_eq!(run.stdout, "a.txt\na.txt\n");
    // The options of the lines around a match have no effect.
    let run = run_bare(root.path(), &["-l", "-C2", "needle", "a.txt"]);
    assert_eq!(run.stdout, "a.txt\n");
}

#[test]
fn criterion_55_c_writes_the_count_of_each_file_also_for_no_match() {
    let root = two_files();
    let run = run_bare(root.path(), &["-c", "needle", "a.txt", "b.txt"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "a.txt:2\nb.txt:0\n");
    // The path shows also for one file, as `grep -H -c`.
    let run = run_bare(root.path(), &["-c", "needle", "a.txt"]);
    assert_eq!(run.stdout, "a.txt:2\n");
    let run = run_bare(root.path(), &["--count", "-C2", "needle", "a.txt"]);
    assert_eq!(run.stdout, "a.txt:2\n");
}

#[test]
fn criterion_55_c_counts_lines_of_a_folder_in_the_order_of_the_scan() {
    let root = folder();
    write(root.path(), "d/b.txt", b"needle needle\n");
    write(root.path(), "d/a.txt", b"x\n");
    write(root.path(), "d/sub/c.txt", b"needle\nneedle\n");
    let run = run_bare(root.path(), &["-c", "needle", "d"]);
    assert_eq!(run.stdout, "d/a.txt:0\nd/b.txt:1\nd/sub/c.txt:2\n");
}

#[test]
fn criterion_55_q_writes_nothing_and_l_and_c_lose_to_it() {
    let root = two_files();
    for args in [
        &["-q", "needle", "a.txt"][..],
        &["--quiet", "needle", "a.txt"],
        &["--silent", "needle", "a.txt"],
        &["-q", "-l", "needle", "a.txt"],
        &["-q", "-c", "needle", "a.txt"],
    ] {
        let run = run_bare(root.path(), args);
        assert_eq!(run.code, Some(0), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
        assert_eq!(run.stderr, "", "{args:?}");
    }
}

#[test]
fn criterion_55_l_wins_over_c() {
    let root = two_files();
    let run = run_bare(root.path(), &["-c", "-l", "needle", "a.txt"]);
    assert_eq!(run.stdout, "a.txt\n");
}

#[test]
fn criterion_55_l_and_c_have_the_color_of_grep() {
    let root = two_files();
    let run = run_bare(root.path(), &["--color=always", "-c", "needle", "a.txt"]);
    assert_eq!(
        run.stdout,
        "\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\x1b[36m\x1b[K:\x1b[m\x1b[K2\n"
    );
    let run = run_bare(root.path(), &["--color=always", "-l", "needle", "a.txt"]);
    assert_eq!(run.stdout, "\x1b[35m\x1b[Ka.txt\x1b[m\x1b[K\n");
}

#[test]
fn criterion_55_l_c_and_q_do_not_work_with_ask_or_json() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    let cases: [(&[&str], &str, &str); 5] = [
        (
            &["-l", "--ask", "a question", "needle", "a.txt"],
            "-l",
            "--ask",
        ),
        (
            &["-c", "--ask", "a question", "needle", "a.txt"],
            "-c",
            "--ask",
        ),
        (
            &["-q", "--ask", "a question", "needle", "a.txt"],
            "-q",
            "--ask",
        ),
        (&["--json", "-c", "needle", "a.txt"], "-c", "--json"),
        (&["--json", "-q", "-l", "needle", "a.txt"], "-q", "--json"),
    ];
    for (args, option, other) in cases {
        let run = run(&server, root.path(), args);
        assert_eq!(run.code, Some(2), "{args:?}");
        if other == "--json" {
            // With `--json`, the error is a document on standard output.
            assert_eq!(run.stderr, "", "{args:?}");
            let document: Value = serde_json::from_str(&run.stdout).expect("a document");
            assert_eq!(
                document["error"]["cause"],
                format!("the option {option} does not work with --json.")
            );
            assert_eq!(
                document["error"]["fix"],
                format!("Remove --json, or remove {option}.")
            );
        } else {
            assert_eq!(run.stdout, "", "{args:?}");
            assert_eq!(
                run.stderr,
                format!(
                    "jag: the option {option} does not work with --ask.\nRemove --ask, or remove {option}.\n"
                ),
                "{args:?}"
            );
        }
    }
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_55_l_c_and_q_make_no_model_call_and_need_no_key() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    for option in ["-l", "-c", "-q"] {
        let run = run(&server, root.path(), &[option, "needle", "a.txt"]);
        assert_eq!(run.code, Some(0), "{option}");
    }
    assert!(server.requests().is_empty());
}

// CRITERION-58: the exit codes.

#[test]
fn criterion_58_a_question_search_exits_0_with_a_good_answer_and_1_with_no_match() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    let run = run(&server, root.path(), &["--ask", "q", "needle", "a.txt"]);
    assert_eq!(run.code, Some(0));
    let run = self::run(&server, root.path(), &["--ask", "q", "absent", "a.txt"]);
    assert_eq!(run.code, Some(1));
}

#[test]
fn criterion_58_each_usage_error_exits_2() {
    let root = two_files();
    let server = FakeServer::constant(0.9);
    for args in [
        &["-A", "x", "needle", "a.txt"][..],
        &["--nope", "needle", "a.txt"],
        &["--include=[", "needle", "a.txt"],
        &["-l", "--ask", "q", "needle", "a.txt"],
        &["-C", "13", "--ask", "q", "needle", "a.txt"],
        &["("],
    ] {
        let run = run(&server, root.path(), args);
        assert_eq!(run.code, Some(2), "{args:?}");
        assert_eq!(run.stdout, "", "{args:?}");
    }
    assert!(server.requests().is_empty());
}

#[test]
fn criterion_58_plain_grep_exits_0_with_a_match_and_1_with_no_match() {
    let root = two_files();
    assert_eq!(run_bare(root.path(), &["needle", "a.txt"]).code, Some(0));
    assert_eq!(run_bare(root.path(), &["zzz", "a.txt"]).code, Some(1));
}

#[test]
fn criterion_58_c_with_no_match_writes_the_counts_and_exits_1() {
    let root = two_files();
    let run = run_bare(root.path(), &["-c", "zzz", "a.txt", "b.txt"]);
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stdout, "a.txt:0\nb.txt:0\n");
}

#[test]
fn criterion_58_l_and_q_with_no_match_write_nothing_and_exit_1() {
    let root = two_files();
    for option in ["-l", "-q"] {
        let run = run_bare(root.path(), &[option, "zzz", "a.txt"]);
        assert_eq!(run.code, Some(1), "{option}");
        assert_eq!(run.stdout, "", "{option}");
        assert_eq!(run.stderr, "", "{option}");
    }
}

#[test]
fn criterion_58_a_file_that_jag_cannot_read_gives_a_warning_and_the_code_does_not_change() {
    let root = folder();
    write(root.path(), "dir/a.txt", b"needle\n");
    write(root.path(), "dir/b.txt", b"needle\n");
    let secret = root.path().join("dir/b.txt");
    std::fs::set_permissions(&secret, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    // A user who can read each file (root) cannot make this test.
    if std::fs::read(&secret).is_ok() {
        return;
    }
    let run = run_bare(root.path(), &["-l", "needle", "dir"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, "dir/a.txt\n");
    assert!(
        run.stderr.contains("cannot read dir/b.txt"),
        "{}",
        run.stderr
    );
}

// FEATURE-09 and FEATURE-13 ("Other features"): the size limit applies to `-l`
// and `-c`, with each line as one whole item.

fn many_files() -> TempDir {
    let root = folder();
    for number in 0..300 {
        write(
            root.path(),
            &format!("file-with-a-long-name-{number}.txt"),
            b"needle\n",
        );
    }
    root
}

#[test]
fn the_size_limit_cuts_the_lines_of_l_and_c_and_gives_exit_8() {
    let root = many_files();
    for option in ["-l", "-c"] {
        let run = run_bare(root.path(), &[option, "--max-chars", "1000", "needle", "."]);
        assert_eq!(run.code, Some(8), "{option}");
        assert!(run.stdout.chars().count() <= 1000, "{option}");
        assert!(run.stdout.ends_with('\n'), "{option}");
        assert!(
            run.stderr.contains("jag: size limit reached: showed "),
            "{option}: {}",
            run.stderr
        );
        assert!(run.stderr.contains("of 300 results"), "{option}");
    }
}

#[test]
fn c_with_no_match_and_a_size_limit_gives_the_notice_and_exit_1() {
    // IF-4: 1 comes before 8 (decision 29).
    let root = many_files();
    let run = run_bare(root.path(), &["-c", "--max-chars", "1000", "zzz", "."]);
    assert_eq!(run.code, Some(1));
    assert!(run.stdout.chars().count() <= 1000);
    assert!(run.stderr.contains("jag: size limit reached: showed "));
}

#[test]
fn with_no_size_limit_l_and_c_write_each_line_and_exit_0() {
    let root = many_files();
    for option in ["-l", "-c"] {
        let run = run_bare(root.path(), &[option, "--no-max-chars", "needle", "."]);
        assert_eq!(run.code, Some(0), "{option}");
        assert_eq!(run.stdout.lines().count(), 300, "{option}");
        assert_eq!(run.stderr, "", "{option}");
    }
}

#[test]
fn the_size_limit_has_no_effect_on_q() {
    let root = many_files();
    let run = run_bare(root.path(), &["-q", "--max-chars", "1000", "needle", "."]);
    assert_eq!(run.code, Some(0));
    assert_eq!((run.stdout.as_str(), run.stderr.as_str()), ("", ""));
}
