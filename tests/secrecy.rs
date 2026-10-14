//! CRITERION-05: jag never shows the key, on every path of the binary.
//!
//! The key text holds `SENTINEL`. The unit tests of the config module cover
//! the paths that need a fake prompter (setup success, "Kept the stored key",
//! and a bad key format in a variable or in the file).

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;

const KEY_FILE: &str = "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\n";

fn jag(home: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("jag"));
    command
        .env("HOME", home)
        .env_remove("JAG_TYPESAFE_API_KEY")
        .env_remove("TYPESAFE_API_KEY")
        .env_remove("JAG_GATEWAY_API_KEY")
        .env_remove("JAG_BASE_URL");
    command
}

fn write_key_file(home: &Path, text: &str, mode: u32) {
    let dir = home.join(".config").join("jag");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.yml");
    std::fs::write(&path, text).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// Run the command and check that no byte of stdout or stderr holds the key.
fn assert_never_shows_the_key(mut command: Command, code: i32) {
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(code));
    for (name, bytes) in [("stdout", &output.stdout), ("stderr", &output.stderr)] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains("SENTINEL"), "{name} shows the key:\n{text}");
    }
}

#[test]
fn setup_with_no_terminal() {
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command
        .arg("--setup")
        .write_stdin("SENTINEL-SENTINEL-SENTINEL\n");
    assert_never_shows_the_key(command, 2);
}

#[test]
fn forget_key_with_a_key_file() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(home.path(), KEY_FILE, 0o600);
    let mut command = jag(home.path());
    command.arg("--forget-key");
    assert_never_shows_the_key(command, 0);
}

#[test]
fn forget_key_with_a_key_variable() {
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command
        .env("JAG_TYPESAFE_API_KEY", "SENTINEL-SENTINEL-SENTINEL")
        .arg("--forget-key");
    assert_never_shows_the_key(command, 0);
}

#[test]
fn forget_key_on_a_bad_yaml_file() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(home.path(), "config:\n  api-key: [SENTINEL\n", 0o600);
    let mut command = jag(home.path());
    command.arg("--forget-key");
    assert_never_shows_the_key(command, 3);
}

#[test]
fn forget_key_on_a_file_with_other_keys() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(
        home.path(),
        "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\nother: 1\n",
        0o600,
    );
    let mut command = jag(home.path());
    command.arg("--forget-key");
    assert_never_shows_the_key(command, 2);
}

#[test]
fn forget_key_on_a_file_that_is_a_link() {
    let home = tempfile::tempdir().unwrap();
    let target_dir = tempfile::tempdir().unwrap();
    let target = target_dir.path().join("real.yml");
    std::fs::write(&target, KEY_FILE).unwrap();
    let dir = home.path().join(".config").join("jag");
    std::fs::create_dir_all(&dir).unwrap();
    std::os::unix::fs::symlink(&target, dir.join("config.yml")).unwrap();
    let mut command = jag(home.path());
    command.arg("--forget-key");
    assert_never_shows_the_key(command, 2);
}

#[test]
fn setup_on_a_key_file_with_mode_644() {
    let home = tempfile::tempdir().unwrap();
    write_key_file(home.path(), KEY_FILE, 0o644);
    let mut command = jag(home.path());
    command.arg("--setup").write_stdin("");
    // With no terminal, setup stops at step 1 (exit 2). The unit tests cover
    // the mode check of step 3 with a fake prompter.
    assert_never_shows_the_key(command, 2);
}

#[test]
fn help_and_version() {
    let home = tempfile::tempdir().unwrap();
    for arg in ["--help", "--version"] {
        let mut command = jag(home.path());
        command
            .env("JAG_TYPESAFE_API_KEY", "SENTINEL-SENTINEL-SENTINEL")
            .arg(arg);
        assert_never_shows_the_key(command, 0);
    }
}

#[test]
fn a_wrong_use_with_a_key_variable_set() {
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command
        .env("TYPESAFE_API_KEY", "SENTINEL-SENTINEL-SENTINEL")
        .args(["--setup", "--forget-key"]);
    assert_never_shows_the_key(command, 2);
}

#[test]
fn a_base_url_with_the_key_in_its_user_name_gives_exit_2() {
    let address = "https://SENTINEL-SENTINEL-SENTINEL@openrouter.ai/api";
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "needle\n").unwrap();
    let search = |command: &mut Command| {
        command
            .env("JAG_GATEWAY_API_KEY", "SENTINEL-SENTINEL-SENTINEL")
            .args(["--ask", "which line?", "needle"])
            .arg(dir.path());
    };
    // As an option.
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command.args(["--base-url", address]);
    search(&mut command);
    assert_never_shows_the_key(command, 2);
    // As a variable.
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command.env("JAG_BASE_URL", address);
    search(&mut command);
    assert_never_shows_the_key(command, 2);
    // In the file.
    let home = tempfile::tempdir().unwrap();
    write_key_file(
        home.path(),
        &format!("config:\n  base-url: \"{address}\"\n"),
        0o600,
    );
    let mut command = jag(home.path());
    search(&mut command);
    assert_never_shows_the_key(command, 2);
}

#[test]
fn a_bad_gateway_key_in_a_variable_or_in_the_file_gives_exit_3() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "needle\n").unwrap();
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command
        .env("JAG_BASE_URL", "https://openrouter.ai/api")
        .env("JAG_GATEWAY_API_KEY", "SENTINEL has a space")
        .args(["--ask", "which line?", "needle"])
        .arg(dir.path());
    assert_never_shows_the_key(command, 3);
    let home = tempfile::tempdir().unwrap();
    write_key_file(
        home.path(),
        "config:\n  gateway-api-key: \"SENTINEL has a space\"\n",
        0o600,
    );
    let mut command = jag(home.path());
    command
        .env("JAG_BASE_URL", "https://openrouter.ai/api")
        .args(["--ask", "which line?", "needle"])
        .arg(dir.path());
    assert_never_shows_the_key(command, 3);
}

#[test]
fn a_model_name_that_is_the_key_is_never_shown() {
    use std::io::{Read, Write};
    // A server that answers 422 to each call.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming().take(1) {
            let mut stream = stream.unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(
                b"HTTP/1.1 422 Unprocessable Entity\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    });
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "needle\n").unwrap();
    let home = tempfile::tempdir().unwrap();
    let mut command = jag(home.path());
    command
        .env("JAG_GATEWAY_API_KEY", "made-up-key")
        .args(["--base-url", &format!("http://127.0.0.1:{port}")])
        .args(["--model", "SENTINEL-SENTINEL-SENTINEL"])
        .args(["--ask", "which line?", "needle"])
        .arg(dir.path());
    assert_never_shows_the_key(command, 6);
    let _ = handle.join();
}
