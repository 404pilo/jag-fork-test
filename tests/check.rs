//! End-to-end tests of `jag --check` against the fake server (FEATURE-02).
//!
//! Each test sets HOME to a temporary folder, uses the made-up key of
//! `common::KEY`, and calls only the fake server on 127.0.0.1. A test with a
//! key variable uses `jag`. A test with no key variable uses `jag_bare` and
//! sets `JAG_BASE_URL`, so that no call goes to the real service.

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use common::fake_server::{FakeServer, Reply};
use common::{KEY, jag, jag_bare};
use tempfile::TempDir;

const USAGE_ERROR: &str = "jag: the command line is not valid. Run jag --help to see the usage.\n";
const KEY_FILE_TEXT: &str = "config:\n  gateway-api-key: SENTINEL-SENTINEL-SENTINEL\n";

const READY: &str = "pass        a key is present (JAG_GATEWAY_API_KEY)\n\
                     pass        the service accepts the key\n\
                     pass        the account has credit\n\
                     pass        the service is reachable\n\
                     pass        other users cannot read the key file (no key file)\n\
                     jag is ready.\n";

fn home() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

/// Write the key file with a mode. Gives its path.
fn key_file(home: &Path, text: &str, mode: u32) -> PathBuf {
    let dir = home.join(".config").join("jag");
    std::fs::create_dir_all(&dir).expect("make folders");
    let path = dir.join("config.yml");
    std::fs::write(&path, text).expect("write file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("set mode");
    path
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run `jag --check` with the key in a variable. No output holds the key,
/// the address, or the text of a body (decision 20).
fn check_with_key(server: &FakeServer, home: &Path, args: &[&str]) -> Run {
    finish(
        jag(home, &server.base_url())
            .env("JAG_MODEL", "jev-1.13.0")
            .arg("--check")
            .args(args),
        server,
    )
}

/// Run `jag --check` with no key variable.
fn check_without_key(server: &FakeServer, home: &Path, args: &[&str]) -> Run {
    finish(
        jag_bare(home)
            .env("JAG_BASE_URL", server.base_url())
            .env("JAG_MODEL", "jev-1.13.0")
            .arg("--check")
            .args(args),
        server,
    )
}

fn finish(command: &mut assert_cmd::Command, server: &FakeServer) -> Run {
    let output = command.output().expect("run jag");
    let run = Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    for text in [&run.stdout, &run.stderr] {
        assert!(!text.contains("SENTINEL"), "{text}");
        assert!(!text.contains("127.0.0.1"), "{text}");
        assert!(!text.contains("SECRET-BODY-TEXT"), "{text}");
    }
    for request in server.requests() {
        assert_eq!(request.method, "POST");
    }
    run
}

fn answer(status: u16) -> FakeServer {
    FakeServer::start(move |_| Reply::status(status).body("SECRET-BODY-TEXT"))
}

fn open_key_file_block(path: &Path) -> String {
    format!(
        "jag: {0} can be read by other users (mode 0644).\nRun `chmod 600 {0}`.\n",
        path.display()
    )
}

const NO_CREDIT_BLOCK: &str = "jag: the gateway account has no credit (HTTP 402).\n\
                               Add credit at the gateway, then try again.\n";

// CRITERION-11: all items pass.
#[test]
fn criterion_11_all_items_pass_and_jag_is_ready() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, READY);
    assert_eq!(run.stderr, "");
}

// CRITERION-09: a private key file is the source, and item 1 names its path.
#[test]
fn criterion_09_a_private_key_file_is_named_as_the_source() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let path = key_file(home.path(), KEY_FILE_TEXT, 0o600);
    let run = check_without_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(0));
    assert_eq!(
        run.stdout,
        format!(
            "pass        a key is present ({})\n\
             pass        the service accepts the key\n\
             pass        the account has credit\n\
             pass        the service is reachable\n\
             pass        other users cannot read the key file\n\
             jag is ready.\n",
            path.display()
        )
    );
    assert_eq!(server.requests().len(), 1);
    // The key from the file goes only into the header.
    assert!(
        server.requests()[0]
            .header("authorization")
            .is_some_and(|value| value.contains(KEY))
    );
}

// CRITERION-09: each item shows pass or fail, and each one is examined.
#[test]
fn criterion_09_each_item_is_examined_with_a_402_and_an_open_key_file() {
    let server = answer(402);
    let home = home();
    let path = key_file(home.path(), KEY_FILE_TEXT, 0o644);
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(5));
    assert_eq!(
        run.stdout,
        "pass        a key is present (JAG_GATEWAY_API_KEY)\n\
         pass        the service accepts the key\n\
         fail        the account has credit\n\
         pass        the service is reachable\n\
         fail        other users cannot read the key file\n"
    );
    // CRITERION-10: two blocks, in item order.
    assert_eq!(
        run.stderr,
        format!("{NO_CREDIT_BLOCK}{}", open_key_file_block(&path))
    );
}

// CRITERION-10 and CRITERION-67: no key (item 1), exit 3, and no call.
#[test]
fn criterion_10_no_key_names_the_problem_and_the_fix_and_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let run = check_without_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "fail        a key is present\n\
         not checked the service accepts the key (jag has no key to send)\n\
         not checked the account has credit (jag has no key to send)\n\
         not checked the service is reachable (jag has no key to send)\n\
         pass        other users cannot read the key file (no key file)\n"
    );
    assert_eq!(
        run.stderr,
        "jag: no gateway key found.\n\
         Get a key from the gateway, then run `jag --setup`,\n\
         or set JAG_GATEWAY_API_KEY.\n"
    );
    assert!(server.requests().is_empty());
}

// CRITERION-10 and CRITERION-67: a refused key (item 2), exit 4.
#[test]
fn criterion_10_a_refused_key_fails_item_2_with_exit_4() {
    let server = answer(401);
    let home = home();
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(4));
    assert_eq!(
        run.stdout,
        "pass        a key is present (JAG_GATEWAY_API_KEY)\n\
         fail        the service accepts the key\n\
         not checked the account has credit (the service did not accept the key)\n\
         pass        the service is reachable\n\
         pass        other users cannot read the key file (no key file)\n"
    );
    assert_eq!(
        run.stderr,
        "jag: the gateway did not accept the key (HTTP 401).\n\
         Check that the key is a key of the gateway, then run jag --setup.\n"
    );
}

// CRITERION-10 and CRITERION-67: no credit (item 3), exit 5.
#[test]
fn criterion_10_no_credit_fails_item_3_with_exit_5() {
    let server = answer(402);
    let home = home();
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(5));
    assert_eq!(run.stderr, NO_CREDIT_BLOCK);
    assert!(run.stdout.contains("fail        the account has credit\n"));
}

// CRITERION-10 and CRITERION-67: the service fails two times (item 4), exit 6.
#[test]
fn criterion_10_a_service_that_fails_fails_item_4_with_exit_6() {
    let server = answer(500);
    let home = home();
    let run = check_with_key(&server, home.path(), &["--retry-wait", "1"]);
    assert_eq!(run.code, Some(6));
    assert_eq!(
        run.stdout,
        "pass        a key is present (JAG_GATEWAY_API_KEY)\n\
         not checked the service accepts the key (the call to the service failed)\n\
         not checked the account has credit (the call to the service failed)\n\
         fail        the service is reachable\n\
         pass        other users cannot read the key file (no key file)\n"
    );
    assert_eq!(
        run.stderr,
        "jag: the gateway failed (HTTP 500).\nTry again later.\n"
    );
    // The call and its one retry.
    assert_eq!(server.requests().len(), 2);
}

// CRITERION-10 and CRITERION-67: a key file that others can read (item 5).
#[test]
fn criterion_10_an_open_key_file_fails_item_5_with_exit_3() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let path = key_file(home.path(), KEY_FILE_TEXT, 0o644);
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "pass        a key is present (JAG_GATEWAY_API_KEY)\n\
         pass        the service accepts the key\n\
         pass        the account has credit\n\
         pass        the service is reachable\n\
         fail        other users cannot read the key file\n"
    );
    assert_eq!(run.stderr, open_key_file_block(&path));
}

// CRITERION-10: an open key file that is the source. Item 1 is not checked.
#[test]
fn criterion_10_an_open_key_file_that_is_the_source_shows_its_block_one_time() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let path = key_file(home.path(), KEY_FILE_TEXT, 0o644);
    let run = check_without_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "not checked a key is present (jag cannot use the key file: see the last item)\n\
         not checked the service accepts the key (jag has no key to send)\n\
         not checked the account has credit (jag has no key to send)\n\
         not checked the service is reachable (jag has no key to send)\n\
         fail        other users cannot read the key file\n"
    );
    assert_eq!(run.stderr, open_key_file_block(&path));
    assert!(server.requests().is_empty());
}

// CRITERION-67: an item 2 failure comes before an item 5 failure.
#[test]
fn criterion_67_a_refused_key_and_an_open_key_file_exit_with_4() {
    let server = answer(401);
    let home = home();
    let path = key_file(home.path(), KEY_FILE_TEXT, 0o644);
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(4));
    assert!(run.stderr.starts_with("jag: the gateway did not accept"));
    assert!(run.stderr.ends_with(&open_key_file_block(&path)));
}

// CRITERION-67: a call that times out fails item 4 with exit 6.
#[test]
fn criterion_67_a_call_that_times_out_fails_item_4_with_exit_6() {
    let server =
        FakeServer::start(|request| Reply::scores(request, |_| 0.8).after(Duration::from_secs(3)));
    let home = home();
    let run = check_with_key(
        &server,
        home.path(),
        &["--timeout", "1", "--retry-wait", "1"],
    );
    assert_eq!(run.code, Some(6));
    assert!(
        run.stdout
            .contains("fail        the service is reachable\n")
    );
    assert_eq!(
        run.stderr,
        "jag: the gateway did not answer in time.\n\
         Try again later, or give each call more time with --timeout.\n"
    );
}

// CRITERION-12: one call, in the form of a search, with one question.
#[test]
fn criterion_12_the_check_makes_one_small_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let run = check_with_key(&server, home.path(), &[]);
    assert_eq!(run.code, Some(0));
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/systemone");
    assert!(request.body.len() < 400, "{} bytes", request.body.len());
    let json = request.json();
    assert_eq!(json["questions"].as_object().map(|map| map.len()), Some(1));
    assert!(
        request
            .header("authorization")
            .is_some_and(|value| value.contains(KEY))
    );
}

// CRITERION-12: a failure that CV-5 tries again adds the one retry.
#[test]
fn criterion_12_a_503_and_then_a_200_give_two_requests_and_exit_0() {
    let count = AtomicUsize::new(0);
    let server = FakeServer::start(move |request| {
        if count.fetch_add(1, Ordering::SeqCst) == 0 {
            Reply::status(503)
        } else {
            Reply::scores(request, |_| 0.8)
        }
    });
    let home = home();
    let run = check_with_key(&server, home.path(), &["--retry-wait", "1"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.stdout, READY);
    assert_eq!(server.requests().len(), 2);
}

#[test]
fn a_check_with_another_mode_is_an_argument_error() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let cases: [&[&str]; 5] = [
        &["--check", "needle"],
        &["--check", "--setup"],
        &["--setup", "--check"],
        &["--check", "--forget-key"],
        &["--check", "--max-scored", "5"],
    ];
    for args in cases {
        let output = jag(home.path(), &server.base_url())
            .args(args)
            .output()
            .expect("run jag");
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            USAGE_ERROR,
            "{args:?}"
        );
        assert!(output.stdout.is_empty(), "{args:?}");
    }
    assert!(server.requests().is_empty());
}

#[test]
fn a_bad_time_option_stops_the_check_with_exit_2_and_no_item_line() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let run = check_with_key(&server, home.path(), &["--timeout", "0"]);
    assert_eq!(run.code, Some(2));
    assert_eq!(run.stdout, "");
    assert!(run.stderr.starts_with("jag: the value of --timeout is not"));
    assert!(server.requests().is_empty());
}

#[test]
fn a_bad_base_url_stops_the_check_with_exit_2_and_no_item_line() {
    let home = home();
    let output = jag(home.path(), "http://example.com")
        .arg("--check")
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("jag: JAG_BASE_URL is not an https address."));
    assert!(!stderr.contains("example.com"));
}

/// The check with the key of the direct route, and no setting variable, so
/// that the settings read needs the key file.
fn check_reading_the_file(server: &FakeServer, home: &Path) -> Run {
    finish(
        jag_bare(home)
            .env("JAG_TYPESAFE_API_KEY", KEY)
            .env("JAG_BASE_URL", server.base_url())
            .arg("--check"),
        server,
    )
}

// FEATURE-21 example 1: a file error of the settings read shows on item 1.
#[test]
fn a_file_with_bad_yaml_fails_item_1_and_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let path = key_file(home.path(), "a: b: [\n", 0o600);
    let run = check_reading_the_file(&server, home.path());
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "fail        a key is present\n\
         not checked the service accepts the key (jag has no key to send)\n\
         not checked the account has credit (jag has no key to send)\n\
         not checked the service is reachable (jag has no key to send)\n\
         pass        other users cannot read the key file\n"
    );
    assert_eq!(
        run.stderr,
        format!(
            "jag: {} is not valid YAML (line 1).\n\
             Fix the file by hand, or remove it and run `jag --setup`.\n",
            path.display()
        )
    );
    assert!(server.requests().is_empty());
}

// FEATURE-21 example 2: an open file shows on item 5 only.
#[test]
fn an_open_file_that_the_settings_need_fails_item_5_and_makes_no_call() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let path = key_file(home.path(), "config:\n  model: jev-1.13.0\n", 0o644);
    let run = check_reading_the_file(&server, home.path());
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "not checked a key is present (jag cannot use the key file: see the last item)\n\
         not checked the service accepts the key (jag has no key to send)\n\
         not checked the account has credit (jag has no key to send)\n\
         not checked the service is reachable (jag has no key to send)\n\
         fail        other users cannot read the key file\n"
    );
    assert_eq!(run.stderr, open_key_file_block(&path));
    assert!(server.requests().is_empty());
}

// FEATURE-21 example 3: the key of the direct route is not a gateway key.
#[test]
fn the_key_of_the_direct_route_does_not_count_on_the_gateway_route() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let run = check_reading_the_file(&server, home.path());
    assert_eq!(run.code, Some(3));
    assert_eq!(
        run.stdout,
        "fail        a key is present\n\
         not checked the service accepts the key (jag has no key to send)\n\
         not checked the account has credit (jag has no key to send)\n\
         not checked the service is reachable (jag has no key to send)\n\
         pass        other users cannot read the key file (no key file)\n"
    );
    assert_eq!(
        run.stderr,
        "jag: no gateway key found.\n\
         Get a key from the gateway, then run `jag --setup`,\n\
         or set JAG_GATEWAY_API_KEY.\n"
    );
    assert!(server.requests().is_empty());
}

#[test]
fn the_check_takes_the_options_for_the_address_and_the_model() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let output = jag_bare(home.path())
        .env("JAG_GATEWAY_API_KEY", KEY)
        .args([
            "--check",
            "--base-url",
            &server.root(),
            "--model",
            "a-model",
        ])
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(0));
    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/v1/systemone");
    assert_eq!(requests[0].json()["model"], "a-model");
}

#[test]
fn a_blank_check_option_stops_the_check_with_exit_2_and_no_item_line() {
    let server = FakeServer::constant(0.8);
    let home = home();
    for (option, start) in [
        (
            "--base-url",
            "jag: the value of --base-url is not an https address.",
        ),
        ("--model", "jag: the value of --model is blank."),
    ] {
        let run = check_with_key(&server, home.path(), &[option, "  "]);
        assert_eq!(run.code, Some(2), "{option}");
        assert_eq!(run.stdout, "", "{option}");
        assert!(run.stderr.starts_with(start), "{}", run.stderr);
    }
    assert!(server.requests().is_empty());
}

#[test]
fn the_address_and_model_options_do_not_go_with_setup_or_forget_key() {
    let server = FakeServer::constant(0.8);
    let home = home();
    for args in [
        ["--setup", "--base-url", "https://openrouter.ai/api"],
        ["--forget-key", "--model", "m"],
    ] {
        let output = jag(home.path(), &server.base_url())
            .args(args)
            .output()
            .expect("run jag");
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert_eq!(String::from_utf8_lossy(&output.stderr), USAGE_ERROR);
    }
}
