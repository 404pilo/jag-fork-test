//! The time of a typical search (STORY-44, CRITERION-65). Only a release
//! build runs it: `cargo test --release --test speed`.
//!
//! The test makes a tree with the size of a medium code repository and runs
//! three checks, one after the other, against the fake server.

mod common;

use std::path::Path;
use std::process::Output;
use std::time::{Duration, Instant};

use common::fake_server::{FakeServer, Reply};
use common::jag;

/// The count of files of a medium code repository (decision 2).
const FILES: usize = 15_000;
const LINES: usize = 240;
/// The most time of the local part of a typical search (decision 4).
const LOCAL: Duration = Duration::from_secs(1);
/// The time of one call in check (b).
const CALL: Duration = Duration::from_secs(2);
/// The target of CRITERION-65.
const TARGET: Duration = Duration::from_secs(5);
const QUESTION: &str = "where does the code retry after an error";
/// The line of each file that holds the term.
const TERM_LINE: usize = 120;

/// The path of file `number`, relative to the tree: 10 files in each folder.
fn relative_path(number: usize) -> String {
    format!("d{:04}/f{}.txt", number / 10, number % 10)
}

/// One file of `LINES` lines of about 40 bytes. Only line `TERM_LINE` holds
/// the term `error`.
fn file_text() -> String {
    let mut text = String::with_capacity(LINES * 42);
    for line in 1..=LINES {
        if line == TERM_LINE {
            text.push_str(&format!("    return handle_error(value_{});\n", line - 1));
        } else {
            text.push_str(&format!(
                "    let value_{line} = add(value_{}, {line});\n",
                line - 1
            ));
        }
    }
    text
}

/// The tree of `FILES` files, in a temporary folder.
fn make_tree() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tree folder");
    let text = file_text();
    for number in 0..FILES {
        let path = root.path().join(relative_path(number));
        if number % 10 == 0 {
            std::fs::create_dir_all(path.parent().expect("a file has a folder")).expect("folder");
        }
        std::fs::write(path, &text).expect("write file");
    }
    root
}

/// Run `jag --ask QUESTION error ARGUMENTS` in `folder`, and give its output
/// with its wall time.
fn search(server: &FakeServer, folder: &Path, arguments: &[String]) -> (Output, Duration) {
    let home = tempfile::tempdir().expect("home folder");
    let mut command = jag(home.path(), &server.base_url());
    command
        .current_dir(folder)
        .args(["--ask", QUESTION, "error"])
        .args(arguments);
    let started = Instant::now();
    let output = command.output().expect("run jag");
    (output, started.elapsed())
}

/// The cost limit stops the search (exit 7), and the search makes 7 calls.
fn assert_typical(name: &str, output: &Output, server: &FakeServer, runs: usize) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(7), "{name}: {stderr}");
    assert_eq!(server.requests().len(), 7 * runs, "{name}: {stderr}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "a speed test runs only in a release build")]
fn a_typical_search_returns_in_under_5_seconds() {
    let tree = make_tree();
    let folder = vec![tree.path().display().to_string()];

    // (a) The service answers at once: the local part takes under 1 s. The
    // first run warms up the file cache.
    let server = FakeServer::constant(0.9);
    let (output, _) = search(&server, tree.path(), &folder);
    assert_typical("warm-up", &output, &server, 1);
    let (output, time) = search(&server, tree.path(), &folder);
    assert_typical("(a)", &output, &server, 2);
    assert!(time < LOCAL, "(a) the local part took {time:?}");

    // (b) The service answers each call after 2 s: two rounds of calls, and
    // the search takes under 5 s.
    let server = FakeServer::start(|request| Reply::scores(request, |_| 0.9).after(CALL));
    let (output, time) = search(&server, tree.path(), &folder);
    assert_typical("(b)", &output, &server, 1);
    assert_eq!(server.most_at_once(), 6, "(b) calls at the same time");
    assert!(time < TARGET, "(b) the search took {time:?}");

    // (c) The tree as the list of its files, as a shell glob gives it. The
    // list has 315,142 bytes with its pointers, under the 1 MiB of macOS.
    let server = FakeServer::constant(0.9);
    let list: Vec<String> = (0..FILES).map(relative_path).collect();
    let (output, time) = search(&server, tree.path(), &list);
    assert_typical("(c)", &output, &server, 1);
    assert!(time < LOCAL, "(c) the search of the list took {time:?}");
}
