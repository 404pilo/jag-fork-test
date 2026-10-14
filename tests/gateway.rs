//! End-to-end tests of FEATURE-21: the calls go to the address and the model
//! name that the user sets (CRITERION-94, 95, 96), and each route has its own
//! key. They use fake servers only, and the made-up key of `common::KEY`.

mod common;

use std::path::Path;
use std::process::Output;

use common::fake_server::FakeServer;
use common::{KEY, jag_bare, write};
use tempfile::TempDir;

const MODEL: &str = "gw-model";
const DEFAULT_MODEL: &str = "jev-1.13.0";

fn folder() -> TempDir {
    let dir = tempfile::tempdir().expect("temporary folder");
    write(dir.path(), "a.txt", b"needle\n");
    dir
}

fn home() -> TempDir {
    tempfile::tempdir().expect("temporary folder")
}

/// Write `config.yml` with mode 0600.
fn config(home: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;
    let dir = home.join(".config").join("jag");
    std::fs::create_dir_all(&dir).expect("make folders");
    let path = dir.join("config.yml");
    std::fs::write(&path, text).expect("write file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("set mode");
}

/// Run a search with no setting, except the ones that the caller gives.
fn search(home: &Path, dir: &Path, variables: &[(&str, &str)], options: &[&str]) -> Output {
    let mut command = jag_bare(home);
    for (name, value) in variables {
        command.env(name, value);
    }
    command
        .args(options)
        .args(["--ask", "which line?", "needle"])
        .arg(dir)
        .output()
        .expect("run jag")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn no_secret(output: &Output) {
    for bytes in [&output.stdout, &output.stderr] {
        let text = text(bytes);
        assert!(!text.contains("SENTINEL"), "{text}");
    }
}

/// The five forms of the address, and the path that each one gives.
fn forms(server: &FakeServer) -> Vec<(String, &'static str)> {
    let root = server.root();
    vec![
        (root.clone(), "/v1/systemone"),
        (format!("{root}/"), "/v1/systemone"),
        (format!("{root}/v1"), "/v1/systemone"),
        (format!("{root}/api"), "/api/v1/systemone"),
        (
            format!("{root}/proxy/typesafe/"),
            "/proxy/typesafe/v1/systemone",
        ),
    ]
}

/// The server got one `POST` to `path` with the model and the gateway key.
fn assert_one_call(server: &FakeServer, path: &str, model: &str) {
    let requests = server.requests();
    assert_eq!(requests.len(), 1, "{path}");
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, path);
    assert_eq!(request.json()["model"], model);
    assert!(
        request
            .header("authorization")
            .is_some_and(|value| value == format!("Bearer {KEY}")),
        "{path}"
    );
}

// CRITERION-94: the options.
#[test]
fn criterion_94_the_options_name_the_gateway_and_the_model() {
    let dir = folder();
    for index in 0..5 {
        let server = FakeServer::constant(0.8);
        let (address, path) = forms(&server).remove(index);
        let home = home();
        let output = search(
            home.path(),
            dir.path(),
            &[("JAG_GATEWAY_API_KEY", KEY)],
            &["--base-url", &address, "--model", MODEL],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{address} {}",
            text(&output.stderr)
        );
        assert_one_call(&server, path, MODEL);
        no_secret(&output);
    }
}

// CRITERION-94: the variables.
#[test]
fn criterion_94_the_variables_name_the_gateway_and_the_model() {
    let dir = folder();
    for index in 0..5 {
        let server = FakeServer::constant(0.8);
        let (address, path) = forms(&server).remove(index);
        let home = home();
        let output = search(
            home.path(),
            dir.path(),
            &[
                ("JAG_GATEWAY_API_KEY", KEY),
                ("JAG_BASE_URL", &address),
                ("JAG_MODEL", MODEL),
            ],
            &[],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{address} {}",
            text(&output.stderr)
        );
        assert_one_call(&server, path, MODEL);
    }
}

// CRITERION-94: the config file.
#[test]
fn criterion_94_the_config_file_names_the_gateway_and_the_model() {
    let dir = folder();
    for index in 0..5 {
        let server = FakeServer::constant(0.8);
        let (address, path) = forms(&server).remove(index);
        let home = home();
        config(
            home.path(),
            &format!("config:\n  base-url: \"{address}\"\n  model: {MODEL}\n"),
        );
        let output = search(
            home.path(),
            dir.path(),
            &[("JAG_GATEWAY_API_KEY", KEY)],
            &[],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{address} {}",
            text(&output.stderr)
        );
        assert_one_call(&server, path, MODEL);
    }
}

#[test]
fn criterion_94_the_gateway_key_of_the_file_is_sent() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    config(
        home.path(),
        &format!(
            "config:\n  gateway-api-key: {KEY}\n  base-url: \"{}\"\n  model: {MODEL}\n",
            server.root()
        ),
    );
    let output = search(home.path(), dir.path(), &[], &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_one_call(&server, "/v1/systemone", MODEL);
}

#[test]
fn the_gateway_key_of_the_variable_wins_over_the_key_of_the_file() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    config(
        home.path(),
        "config:\n  gateway-api-key: another-made-up-key\n",
    );
    let output = search(
        home.path(),
        dir.path(),
        &[
            ("JAG_GATEWAY_API_KEY", KEY),
            ("JAG_BASE_URL", &server.root()),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_one_call(&server, "/v1/systemone", DEFAULT_MODEL);
}

#[test]
fn criterion_94_the_check_takes_the_address_and_the_model_options() {
    let server = FakeServer::constant(0.8);
    let home = home();
    let output = jag_bare(home.path())
        .env("JAG_GATEWAY_API_KEY", KEY)
        .args(["--check", "--base-url", &server.root(), "--model", MODEL])
        .output()
        .expect("run jag");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_one_call(&server, "/v1/systemone", MODEL);
    assert!(text(&output.stdout).ends_with("jag is ready.\n"));
}

// CRITERION-96: the base URL has its own order.
#[test]
fn criterion_96_an_option_wins_over_a_variable_and_a_variable_over_the_file() {
    let dir = folder();
    let key = [("JAG_GATEWAY_API_KEY", KEY)];

    let (a, b) = (FakeServer::constant(0.8), FakeServer::constant(0.8));
    let home1 = home();
    let output = search(
        home1.path(),
        dir.path(),
        &[key[0], ("JAG_BASE_URL", &b.root())],
        &["--base-url", &a.root()],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!((a.requests().len(), b.requests().len()), (1, 0));

    let (a, b) = (FakeServer::constant(0.8), FakeServer::constant(0.8));
    let home2 = home();
    config(
        home2.path(),
        &format!("config:\n  base-url: \"{}\"\n", b.root()),
    );
    let output = search(
        home2.path(),
        dir.path(),
        &[key[0], ("JAG_BASE_URL", &a.root())],
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!((a.requests().len(), b.requests().len()), (1, 0));

    let a = FakeServer::constant(0.8);
    let home3 = home();
    config(
        home3.path(),
        &format!("config:\n  base-url: \"{}\"\n", a.root()),
    );
    let output = search(home3.path(), dir.path(), &key, &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(a.requests().len(), 1);
}

/// The variables, the options, `config.model`, and the model that jag sends.
type ModelCase<'a> = (
    &'a [(&'a str, &'a str)],
    &'a [&'a str],
    Option<&'a str>,
    &'a str,
);

// CRITERION-96: the model has its own order, and the default is the pinned
// model.
#[test]
fn criterion_96_the_model_comes_from_the_first_source_that_is_set() {
    let dir = folder();
    let cases: [ModelCase; 4] = [
        (
            &[("JAG_MODEL", "m-variable")],
            &["--model", "m-option"],
            Some("m-file"),
            "m-option",
        ),
        (
            &[("JAG_MODEL", "m-variable")],
            &[],
            Some("m-file"),
            "m-variable",
        ),
        (&[], &[], Some("m-file"), "m-file"),
        (&[], &[], None, DEFAULT_MODEL),
    ];
    for (variables, options, file, want) in cases {
        let server = FakeServer::constant(0.8);
        let home = home();
        if let Some(model) = file {
            config(home.path(), &format!("config:\n  model: {model}\n"));
        }
        let mut all = vec![("JAG_GATEWAY_API_KEY", KEY), ("JAG_BASE_URL", "")];
        let root = server.root();
        all[1].1 = &root;
        all.extend_from_slice(variables);
        let output = search(home.path(), dir.path(), &all, options);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{want} {}",
            text(&output.stderr)
        );
        assert_one_call(&server, "/v1/systemone", want);
    }
}

#[test]
fn criterion_96_each_setting_has_its_own_order() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    config(home.path(), "config:\n  model: m-file\n");
    let output = search(
        home.path(),
        dir.path(),
        &[("JAG_GATEWAY_API_KEY", KEY)],
        &["--base-url", &server.root()],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_one_call(&server, "/v1/systemone", "m-file");
}

#[test]
fn criterion_96_a_source_that_wins_hides_a_bad_lower_value() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home1 = home();
    let output = search(
        home1.path(),
        dir.path(),
        &[
            ("JAG_GATEWAY_API_KEY", KEY),
            ("JAG_BASE_URL", "http://example.com"),
        ],
        &["--base-url", &server.root()],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));

    let home2 = home();
    config(home2.path(), "config:\n  base-url: \"ftp://example.com\"\n");
    let output = search(
        home2.path(),
        dir.path(),
        &[
            ("JAG_GATEWAY_API_KEY", KEY),
            ("JAG_BASE_URL", &server.root()),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(server.requests().len(), 2);
}

// The key of each route: the key of the other route does not count.
#[test]
fn a_typesafe_key_is_not_a_gateway_key() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    config(home.path(), &format!("config:\n  api-key: {KEY}\n"));
    let variables = [
        ("JAG_TYPESAFE_API_KEY", KEY),
        ("TYPESAFE_API_KEY", KEY),
        ("JAG_BASE_URL", &server.root()),
    ];
    let output = search(home.path(), dir.path(), &variables, &[]);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        text(&output.stderr),
        "jag: no gateway key found.\n\
         Get a key from the gateway, then run `jag --setup`,\n\
         or set JAG_GATEWAY_API_KEY.\n"
    );
    assert!(server.requests().is_empty());
    no_secret(&output);

    // With the option, the fix names only the variable.
    let output = search(
        home.path(),
        dir.path(),
        &variables[..2],
        &["--base-url", &server.root()],
    );
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        text(&output.stderr),
        "jag: no gateway key found.\nSet JAG_GATEWAY_API_KEY to the key of the gateway.\n"
    );
    assert!(server.requests().is_empty());
}

#[test]
fn a_gateway_key_is_not_a_typesafe_key() {
    let dir = folder();
    let home = home();
    config(home.path(), "config:\n  gateway-api-key: a-made-up-key\n");
    let output = search(
        home.path(),
        dir.path(),
        &[("JAG_GATEWAY_API_KEY", KEY)],
        &[],
    );
    assert_eq!(output.status.code(), Some(3));
    assert!(text(&output.stderr).starts_with("jag: no TypeSafe API key found."));
}

#[test]
fn a_bad_key_of_the_other_route_is_never_read() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    let output = search(
        home.path(),
        dir.path(),
        &[
            ("JAG_GATEWAY_API_KEY", KEY),
            ("JAG_TYPESAFE_API_KEY", "has a space"),
            ("JAG_BASE_URL", &server.root()),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
}

#[test]
fn a_bad_gateway_key_gives_exit_3_and_no_output_holds_it() {
    let dir = folder();
    let server = FakeServer::constant(0.8);
    let home = home();
    let output = search(
        home.path(),
        dir.path(),
        &[
            ("JAG_GATEWAY_API_KEY", "SENTINEL has a space"),
            ("JAG_BASE_URL", &server.root()),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(3));
    no_secret(&output);
    assert!(server.requests().is_empty());
}
