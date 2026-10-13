//! End-to-end tests of `--setup` and `--forget-key`.
//!
//! Each test sets HOME to a temp folder and removes the key variables, so a
//! key on the test machine does not change the result. The keys are made up.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use jag::config::{ConfigError, Env, RealFs, find_key};
use predicates::prelude::*;

fn jag(home: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("jag"));
    command
        .env("HOME", home)
        .env_remove("JAG_TYPESAFE_API_KEY")
        .env_remove("TYPESAFE_API_KEY");
    command
}

fn key_path(home: &Path) -> PathBuf {
    home.join(".config").join("jag").join("config.yml")
}

fn write_key_file(home: &Path, text: &str) {
    let path = key_path(home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

// CRITERION-72: setup with no terminal fails clearly.
#[test]
fn setup_with_no_terminal_exits_2_and_stores_nothing() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .arg("--setup")
        .write_stdin("")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("--setup needs a terminal"))
        .stderr(predicate::str::contains("JAG_TYPESAFE_API_KEY"));
    assert!(!key_path(home.path()).exists());
}

#[test]
fn setup_with_a_piped_key_does_not_read_it() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .arg("--setup")
        .write_stdin("SENTINEL-SENTINEL-SENTINEL\n")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty());
    assert!(!key_path(home.path()).exists());
}

// CRITERION-04: one command removes the stored key.
#[test]
fn forget_key_removes_the_stored_key() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(
        home.path(),
        "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\n",
    );
    jag(home.path())
        .arg("--forget-key")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("Removed the stored key"));
    assert!(!key_path(home.path()).exists());

    let vars = |_: &str| None;
    let env = Env {
        vars: &vars,
        home: Some(home.path().to_owned()),
    };
    assert_eq!(
        find_key(&env, &RealFs).unwrap_err(),
        ConfigError::KeyMissing
    );
}

#[test]
fn forget_key_with_no_file_exits_0() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .arg("--forget-key")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("No stored key"));
}

#[test]
fn forget_key_notes_a_variable_that_is_still_set() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(home.path(), "config:\n  api-key: from-file\n");
    jag(home.path())
        .env("TYPESAFE_API_KEY", "SENTINEL-SENTINEL-SENTINEL")
        .arg("--forget-key")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("TYPESAFE_API_KEY is still set"))
        .stdout(predicate::str::contains("SENTINEL").not());
}

#[test]
fn setup_and_forget_key_are_exclusive() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .args(["--setup", "--forget-key"])
        .assert()
        .code(2);
    jag(home.path()).args(["--setup", "term"]).assert().code(2);
}

#[test]
fn no_option_prints_the_usage_and_exits_2() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn help_and_version_work() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .arg("--help")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("--setup"))
        .stdout(predicate::str::contains("--forget-key"));
    jag(home.path())
        .arg("--version")
        .assert()
        .code(0)
        .stdout(predicate::str::starts_with("jag "));
}

#[test]
fn no_option_takes_the_key() {
    let home = tempfile::tempdir().unwrap();
    jag(home.path())
        .args(["--setup", "--key", "SENTINEL-SENTINEL-SENTINEL"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("SENTINEL").not());
}
