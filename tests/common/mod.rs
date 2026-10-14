//! Helpers that the integration tests share.

#![allow(dead_code)]

pub mod fake_server;

use std::path::Path;

use assert_cmd::Command;

/// A made-up key with low entropy, so that the private-data check passes.
pub const KEY: &str = "SENTINEL-SENTINEL-SENTINEL";

/// The variables that `ureq` reads for a proxy.
const PROXY_VARIABLES: [&str; 8] = [
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
];

/// The `jag` command with no key and no setting of the model service. `HOME`
/// is a folder of the test, so that no real key file is read.
pub fn jag_bare(home: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("jag"));
    command
        .env("HOME", home)
        .env_remove("JAG_TYPESAFE_API_KEY")
        .env_remove("TYPESAFE_API_KEY")
        .env_remove("JAG_GATEWAY_API_KEY")
        .env_remove("JAG_BASE_URL")
        .env_remove("JAG_MODEL")
        .env_remove("JAG_TIMEOUT")
        .env_remove("JAG_RETRY_WAIT")
        .env_remove("JAG_MAX_SCORED")
        .env_remove("JAG_MAX_CHARS");
    for name in PROXY_VARIABLES {
        command.env_remove(name);
    }
    command
}

/// The `jag` command with the made-up gateway key and the address of a fake
/// server. The address is not the TypeSafe address, so the route is the
/// gateway route, and the key is a gateway key.
pub fn jag(home: &Path, base_url: &str) -> Command {
    let mut command = jag_bare(home);
    command
        .env("JAG_GATEWAY_API_KEY", KEY)
        .env("JAG_BASE_URL", base_url);
    command
}

/// A `std::process::Command` with the variables of `jag`: the made-up gateway
/// key and the address of a fake server. A test that sends a signal, or that gives
/// its own standard output, uses it, because `assert_cmd` cannot do that.
pub fn jag_std(home: &Path, base_url: &str) -> std::process::Command {
    let mut command = std::process::Command::new(assert_cmd::cargo::cargo_bin!("jag"));
    command
        .env("HOME", home)
        .env("JAG_GATEWAY_API_KEY", KEY)
        .env("JAG_BASE_URL", base_url)
        .env_remove("JAG_TYPESAFE_API_KEY")
        .env_remove("TYPESAFE_API_KEY")
        .env_remove("JAG_MODEL")
        .env_remove("JAG_TIMEOUT")
        .env_remove("JAG_RETRY_WAIT")
        .env_remove("JAG_MAX_SCORED")
        .env_remove("JAG_MAX_CHARS");
    for name in PROXY_VARIABLES {
        command.env_remove(name);
    }
    command
}

/// Write a file under `root`, with its folders.
pub fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("make folders");
    std::fs::write(path, bytes).expect("write file");
}
