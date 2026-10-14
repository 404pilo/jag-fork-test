//! C-check: the readiness check (FEATURE-02).
//!
//! Five items, in the order of DECISION-04: a key is present, the service
//! accepts the key, the account has credit, the service is reachable, and
//! other users cannot read the key file. One small call to the service
//! examines items 2, 3, and 4. The exit code is the code of the first item
//! that fails.

use crate::config::{ApiKey, ConfigError, ConfigFile, Env, KeySource, find_key, key_file_mode};
use crate::failure::Failure;
use crate::scorer::{ScoreError, ScoreSettings};

const ITEM_1: &str = "a key is present";
const ITEM_2: &str = "the service accepts the key";
const ITEM_3: &str = "the account has credit";
const ITEM_4: &str = "the service is reachable";
const ITEM_5: &str = "other users cannot read the key file";

const NO_KEY_NOTE: &str = " (jag has no key to send)";
const CALL_FAILED_NOTE: &str = " (the call to the service failed)";
const KEY_REFUSED_NOTE: &str = " (the service did not accept the key)";
const KEY_FILE_NOTE: &str = " (jag cannot use the key file: see the last item)";

/// The words of the status field are `pass`, `fail`, and `not checked`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    /// jag cannot examine the item, because an item before it failed.
    NotChecked,
}

impl Status {
    fn word(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::NotChecked => "not checked",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub status: Status,
    /// The text after the status, with its note in brackets.
    pub text: String,
    /// `Some` only for `Fail`. Its `exit_code` is the code of the item.
    pub failure: Option<Failure>,
}

impl Item {
    fn pass(text: &str, note: &str) -> Item {
        Item {
            status: Status::Pass,
            text: format!("{text}{note}"),
            failure: None,
        }
    }

    fn fail(text: &str, failure: Failure) -> Item {
        Item {
            status: Status::Fail,
            text: text.to_owned(),
            failure: Some(failure),
        }
    }

    fn not_checked(text: &str, note: &str) -> Item {
        Item {
            status: Status::NotChecked,
            text: format!("{text}{note}"),
            failure: None,
        }
    }
}

/// The result of the check: the five items, in the order of DECISION-04.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub items: [Item; 5],
}

impl Report {
    /// The lines for standard output, and `jag is ready.` when all items
    /// pass.
    pub fn lines(&self) -> String {
        let mut text = String::new();
        for item in &self.items {
            text.push_str(&format!("{:<12}{}\n", item.status.word(), item.text));
        }
        if self.items.iter().all(|item| item.status == Status::Pass) {
            text.push_str("jag is ready.\n");
        }
        text
    }

    /// The failures, in item order. The first is the cause of the exit code.
    pub fn failures(&self) -> impl Iterator<Item = &Failure> {
        self.items.iter().filter_map(|item| item.failure.as_ref())
    }

    /// 0 when all items pass. Else the code of the first item that fails, in
    /// the order of DECISION-04 (decision 13).
    pub fn exit_code(&self) -> u8 {
        self.failures()
            .next()
            .map_or(0, |failure| failure.exit_code)
    }
}

/// The note of an item 1 that passes: the source of the key (DECISION-03).
fn source_note(source: &KeySource) -> String {
    match source {
        KeySource::JagVariable => format!(" ({})", crate::config::JAG_VARIABLE),
        KeySource::TypeSafeVariable => format!(" ({})", crate::config::TYPESAFE_VARIABLE),
        KeySource::GatewayVariable => format!(" ({})", crate::config::GATEWAY_VARIABLE),
        KeySource::ConfigFile(path) | KeySource::GatewayConfigFile(path) => {
            format!(" ({})", path.display())
        }
    }
}

/// Examine the five items. `settings` is the result of the read of the
/// settings: a file error shows on item 1 or item 5. `call` makes the one
/// call to the service.
pub fn run(
    env: &Env,
    file: &ConfigFile,
    settings: Result<ScoreSettings, ConfigError>,
    call: &dyn Fn(&ApiKey, &ScoreSettings) -> Result<(), ScoreError>,
) -> Report {
    // Item 5.
    let (item_5, key_file_failed) = match key_file_mode(env, file.fs()) {
        Ok(None) => (Item::pass(ITEM_5, " (no key file)"), false),
        Ok(Some((_, mode))) if mode & 0o077 == 0 => (Item::pass(ITEM_5, ""), false),
        Ok(Some((path, mode))) => (
            Item::fail(ITEM_5, ConfigError::KeyFileOpen { path, mode }.into()),
            true,
        ),
        Err(error) => (Item::fail(ITEM_5, error.into()), true),
    };

    // Item 1. jag does not read a key file that it refuses (decision 10).
    // With a file error of the settings, jag does not know the route.
    let no_key = |item| {
        let not_checked = |text| Item::not_checked(text, NO_KEY_NOTE);
        (
            item,
            [
                not_checked(ITEM_2),
                not_checked(ITEM_3),
                not_checked(ITEM_4),
            ],
        )
    };
    let (item_1, items_234) = match settings {
        Err(_) if key_file_failed => no_key(Item::not_checked(ITEM_1, KEY_FILE_NOTE)),
        Err(error) => no_key(Item::fail(ITEM_1, error.into())),
        Ok(settings) => {
            if key_file_failed && env.set_key_variable(settings.route).is_none() {
                no_key(Item::not_checked(ITEM_1, KEY_FILE_NOTE))
            } else {
                match find_key(env, file, settings.route, &settings.base_url_origin) {
                    Ok((key, source)) => (
                        Item::pass(ITEM_1, &source_note(&source)),
                        call_items(&key, &settings, call),
                    ),
                    Err(error) => no_key(Item::fail(ITEM_1, error.into())),
                }
            }
        }
    };
    let [item_2, item_3, item_4] = items_234;
    Report {
        items: [item_1, item_2, item_3, item_4, item_5],
    }
}

/// Items 2, 3, and 4 from the one call (see "The call results" of the spec).
/// A 401 or a 402 comes from the service, so the service is reachable.
fn call_items(
    key: &ApiKey,
    settings: &ScoreSettings,
    call: &dyn Fn(&ApiKey, &ScoreSettings) -> Result<(), ScoreError>,
) -> [Item; 3] {
    let failure = |error: ScoreError| error.failure(settings);
    match call(key, settings) {
        Ok(()) => [
            Item::pass(ITEM_2, ""),
            Item::pass(ITEM_3, ""),
            Item::pass(ITEM_4, ""),
        ],
        Err(error @ ScoreError::KeyNotAccepted { .. }) => [
            Item::fail(ITEM_2, failure(error)),
            Item::not_checked(ITEM_3, KEY_REFUSED_NOTE),
            Item::pass(ITEM_4, ""),
        ],
        Err(ScoreError::NoCredit) => [
            Item::pass(ITEM_2, ""),
            Item::fail(ITEM_3, failure(ScoreError::NoCredit)),
            Item::pass(ITEM_4, ""),
        ],
        Err(error) => [
            Item::not_checked(ITEM_2, CALL_FAILED_NOTE),
            Item::not_checked(ITEM_3, CALL_FAILED_NOTE),
            Item::fail(ITEM_4, failure(error)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::config::testing::{SENTINEL_KEY, vars};
    use crate::config::{Fs, OpenFile, SettingOptions, model_settings};

    /// A key file that is a fake. `mode` is `None` for no file.
    struct FakeFs {
        file: Option<(u32, String)>,
        special: bool,
    }

    struct FakeFile(u32, String);

    impl OpenFile for FakeFile {
        fn mode(&self) -> io::Result<u32> {
            Ok(self.0)
        }
        fn read_to_string(&mut self) -> io::Result<String> {
            Ok(self.1.clone())
        }
    }

    impl Fs for FakeFs {
        fn open(&self, _: &Path) -> io::Result<Box<dyn OpenFile>> {
            match &self.file {
                Some((mode, text)) => Ok(Box::new(FakeFile(*mode, text.clone()))),
                None => Err(io::ErrorKind::NotFound.into()),
            }
        }
        fn is_symlink(&self, _: &Path) -> io::Result<bool> {
            Ok(false)
        }
        fn is_special(&self, _: &Path) -> io::Result<bool> {
            Ok(self.special)
        }
        fn create_dir_all(&self, _: &Path) -> io::Result<()> {
            unreachable!("the check changes no file")
        }
        fn create_private_dir(&self, _: &Path) -> io::Result<()> {
            unreachable!("the check changes no file")
        }
        fn create_private_file(&self, _: &Path, _: &str) -> io::Result<()> {
            unreachable!("the check changes no file")
        }
        fn rename(&self, _: &Path, _: &Path) -> io::Result<()> {
            unreachable!("the check changes no file")
        }
        fn remove_file(&self, _: &Path) -> io::Result<()> {
            unreachable!("the check changes no file")
        }
    }

    const KEY_FILE_TEXT: &str = "config:\n  api-key: SENTINEL-SENTINEL-SENTINEL\n";

    fn no_file() -> FakeFs {
        FakeFs {
            file: None,
            special: false,
        }
    }

    fn file(mode: u32, text: &str) -> FakeFs {
        FakeFs {
            file: Some((mode, text.to_owned())),
            special: false,
        }
    }

    /// Run the check. Gives the report and the count of calls.
    fn check(
        pairs: &[(&str, &str)],
        home: bool,
        fs: &FakeFs,
        result: Result<(), ScoreError>,
    ) -> (Report, usize) {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.then(|| PathBuf::from("/home/user")),
        };
        let calls = Cell::new(0);
        let file = ConfigFile::new(&env, fs);
        let settings =
            model_settings(&env, &file, &SettingOptions::default()).map(ScoreSettings::new);
        let report = run(&env, &file, settings, &|_, _| {
            calls.set(calls.get() + 1);
            result.clone()
        });
        (report, calls.get())
    }

    fn statuses(report: &Report) -> Vec<Status> {
        report.items.iter().map(|item| item.status).collect()
    }

    use Status::{Fail, NotChecked, Pass};

    /// The key, and the two settings that need no file read.
    const GOOD: &[(&str, &str)] = &[
        ("JAG_TYPESAFE_API_KEY", SENTINEL_KEY),
        ("JAG_BASE_URL", "https://api.typesafe.ai/v1"),
        ("JAG_MODEL", "jev-1.13.0"),
    ];

    /// A gateway, the gateway key, and the model: no file read.
    const GATEWAY: &[(&str, &str)] = &[
        ("JAG_GATEWAY_API_KEY", SENTINEL_KEY),
        ("JAG_BASE_URL", "https://openrouter.ai/api"),
        ("JAG_MODEL", "~typesafe/jev-latest"),
    ];

    #[test]
    fn a_key_in_a_variable_and_no_key_file_pass_all_items() {
        let (report, calls) = check(GOOD, true, &no_file(), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert_eq!(calls, 1);
        assert_eq!(report.exit_code(), 0);
        assert_eq!(
            report.items[0].text,
            "a key is present (JAG_TYPESAFE_API_KEY)"
        );
        assert_eq!(
            report.items[4].text,
            "other users cannot read the key file (no key file)"
        );
    }

    #[test]
    fn no_home_gives_no_key_file() {
        let (report, _) = check(GOOD, false, &no_file(), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert!(report.items[4].text.ends_with("(no key file)"));
    }

    #[test]
    fn the_other_variable_is_named_as_the_source() {
        let pairs = [
            ("TYPESAFE_API_KEY", SENTINEL_KEY),
            ("JAG_BASE_URL", "https://api.typesafe.ai/v1"),
            ("JAG_MODEL", "jev-1.13.0"),
        ];
        let (report, _) = check(&pairs, true, &no_file(), Ok(()));
        assert_eq!(report.items[0].text, "a key is present (TYPESAFE_API_KEY)");
    }

    #[test]
    fn a_private_key_file_is_the_source_and_item_5_has_no_note() {
        let (report, _) = check(&[], true, &file(0o600, KEY_FILE_TEXT), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert_eq!(
            report.items[0].text,
            "a key is present (/home/user/.config/jag/config.yml)"
        );
        assert_eq!(report.items[4].text, "other users cannot read the key file");
    }

    #[test]
    fn a_key_in_a_variable_and_a_private_key_file_pass() {
        let (report, calls) = check(GOOD, true, &file(0o600, "not: yaml: ["), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert_eq!(calls, 1);
    }

    #[test]
    fn a_key_in_a_variable_and_an_open_key_file_fail_item_5_only() {
        let (report, calls) = check(GOOD, true, &file(0o644, KEY_FILE_TEXT), Ok(()));
        assert_eq!(statuses(&report), [Pass, Pass, Pass, Pass, Fail]);
        assert_eq!(calls, 1);
        assert_eq!(report.exit_code(), 3);
        let failure = report.items[4].failure.as_ref().expect("a failure");
        assert!(
            failure
                .cause
                .contains("can be read by other users (mode 0644)")
        );
        assert!(failure.fix.contains("chmod 600"));
    }

    #[test]
    fn a_bad_key_in_a_variable_fails_item_1_and_makes_no_call() {
        let pairs = [("JAG_TYPESAFE_API_KEY", "has a space")];
        let (report, calls) = check(&pairs, true, &no_file(), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.exit_code(), 3);
    }

    #[test]
    fn a_bad_key_in_a_variable_and_an_open_key_file_fail_items_1_and_5() {
        let pairs = [
            ("JAG_TYPESAFE_API_KEY", "has a space"),
            ("JAG_BASE_URL", "https://api.typesafe.ai/v1"),
            ("JAG_MODEL", "jev-1.13.0"),
        ];
        let (report, calls) = check(&pairs, true, &file(0o644, KEY_FILE_TEXT), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Fail]
        );
        assert_eq!(calls, 0);
    }

    #[test]
    fn no_key_fails_item_1_and_makes_no_call() {
        let (report, calls) = check(&[], true, &no_file(), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.exit_code(), 3);
        for item in &report.items[1..4] {
            assert!(item.text.ends_with("(jag has no key to send)"));
        }
    }

    #[test]
    fn no_home_and_no_variable_fail_item_1() {
        let (report, calls) = check(&[], false, &no_file(), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(calls, 0);
    }

    #[test]
    fn a_private_key_file_with_no_key_fails_item_1() {
        let (report, _) = check(&[], true, &file(0o600, "config: {}\n"), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        let failure = report.items[0].failure.as_ref().expect("a failure");
        assert!(failure.cause.contains("no TypeSafe API key found"));
    }

    #[test]
    fn a_private_key_file_with_bad_yaml_fails_item_1() {
        let (report, _) = check(&[], true, &file(0o600, "a: b: ["), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(report.exit_code(), 3);
    }

    #[test]
    fn a_private_key_file_with_a_bad_key_fails_item_1() {
        let text = "config:\n  api-key: \"has a space\"\n";
        let (report, _) = check(&[], true, &file(0o600, text), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
    }

    #[test]
    fn an_open_key_file_that_is_the_source_is_not_checked_at_item_1() {
        let (report, calls) = check(&[], true, &file(0o644, KEY_FILE_TEXT), Ok(()));
        assert_eq!(
            statuses(&report),
            [NotChecked, NotChecked, NotChecked, NotChecked, Fail]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.exit_code(), 3);
        assert_eq!(report.failures().count(), 1);
        assert!(
            report.items[0]
                .text
                .ends_with("(jag cannot use the key file: see the last item)")
        );
        let all = format!("{report:?}");
        assert!(!all.contains(SENTINEL_KEY));
    }

    #[test]
    fn a_key_file_that_is_not_a_regular_file_fails_item_5() {
        let fs = FakeFs {
            file: Some((0o600, KEY_FILE_TEXT.to_owned())),
            special: true,
        };
        let (report, calls) = check(&[], true, &fs, Ok(()));
        assert_eq!(
            statuses(&report),
            [NotChecked, NotChecked, NotChecked, NotChecked, Fail]
        );
        assert_eq!(calls, 0);
        let failure = report.items[4].failure.as_ref().expect("a failure");
        assert!(failure.cause.contains("is not a regular file"));
    }

    #[test]
    fn a_refused_key_fails_item_2() {
        let (report, calls) = check(
            GOOD,
            true,
            &no_file(),
            Err(ScoreError::KeyNotAccepted { status: 401 }),
        );
        assert_eq!(statuses(&report), [Pass, Fail, NotChecked, Pass, Pass]);
        assert_eq!(calls, 1);
        assert_eq!(report.exit_code(), 4);
        assert!(
            report.items[2]
                .text
                .ends_with("(the service did not accept the key)")
        );
    }

    #[test]
    fn no_credit_fails_item_3() {
        let (report, _) = check(GOOD, true, &no_file(), Err(ScoreError::NoCredit));
        assert_eq!(statuses(&report), [Pass, Pass, Fail, Pass, Pass]);
        assert_eq!(report.exit_code(), 5);
    }

    #[test]
    fn each_failure_of_the_service_fails_item_4() {
        let errors = [
            ScoreError::TimedOut,
            ScoreError::CannotConnect,
            ScoreError::RateLimited,
            ScoreError::ServerFailed { status: 503 },
            ScoreError::Redirect,
            ScoreError::UnexpectedStatus { status: 418 },
            ScoreError::BadResponse,
        ];
        for error in errors {
            let (report, calls) = check(GOOD, true, &no_file(), Err(error.clone()));
            assert_eq!(
                statuses(&report),
                [Pass, NotChecked, NotChecked, Fail, Pass],
                "{error:?}"
            );
            assert_eq!(calls, 1);
            assert_eq!(report.exit_code(), 6);
            assert!(
                report.items[1]
                    .text
                    .ends_with("(the call to the service failed)")
            );
        }
    }

    #[test]
    fn the_first_item_that_fails_gives_the_exit_code() {
        // Item 2 and item 5 fail: the code is 4.
        let (report, _) = check(
            GOOD,
            true,
            &file(0o644, KEY_FILE_TEXT),
            Err(ScoreError::KeyNotAccepted { status: 403 }),
        );
        assert_eq!(statuses(&report), [Pass, Fail, NotChecked, Pass, Fail]);
        assert_eq!(report.exit_code(), 4);
        // Item 3 and item 5 fail: the code is 5.
        let (report, _) = check(
            GOOD,
            true,
            &file(0o644, KEY_FILE_TEXT),
            Err(ScoreError::NoCredit),
        );
        assert_eq!(report.exit_code(), 5);
        // The blocks are in item order.
        let codes: Vec<u8> = report.failures().map(|failure| failure.exit_code).collect();
        assert_eq!(codes, [5, 3]);
    }

    #[test]
    fn a_gateway_key_variable_is_the_source_on_the_gateway_route() {
        let (report, calls) = check(GATEWAY, true, &no_file(), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert_eq!(calls, 1);
        assert_eq!(
            report.items[0].text,
            "a key is present (JAG_GATEWAY_API_KEY)"
        );
    }

    #[test]
    fn the_key_of_the_other_route_is_not_a_key_of_the_gateway() {
        let pairs = [
            ("JAG_TYPESAFE_API_KEY", SENTINEL_KEY),
            ("JAG_BASE_URL", "https://openrouter.ai/api"),
            ("JAG_MODEL", "~typesafe/jev-latest"),
        ];
        let (report, calls) = check(&pairs, true, &no_file(), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.exit_code(), 3);
        let failure = report.items[0].failure.as_ref().expect("a failure");
        assert!(failure.cause.contains("no gateway key found"));
    }

    #[test]
    fn a_gateway_key_in_the_file_names_the_file() {
        let text = "config:\n  gateway-api-key: SENTINEL-SENTINEL-SENTINEL\n  base-url: https://openrouter.ai/api\n  model: m\n";
        let (report, calls) = check(&[], true, &file(0o600, text), Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
        assert_eq!(calls, 1);
        assert_eq!(
            report.items[0].text,
            "a key is present (/home/user/.config/jag/config.yml)"
        );
    }

    #[test]
    fn a_failure_of_the_gateway_has_the_text_of_the_gateway() {
        let (report, _) = check(
            GATEWAY,
            true,
            &no_file(),
            Err(ScoreError::KeyNotAccepted { status: 401 }),
        );
        let failure = report.items[1].failure.as_ref().expect("a failure");
        assert!(failure.cause.contains("the gateway did not accept the key"));
    }

    #[test]
    fn a_file_error_of_the_settings_fails_item_1_when_item_5_passes() {
        // The key variable is set, but the route needs the file, and the YAML
        // is not valid.
        let pairs = [("JAG_TYPESAFE_API_KEY", SENTINEL_KEY)];
        let (report, calls) = check(&pairs, true, &file(0o600, "a: b: ["), Ok(()));
        assert_eq!(
            statuses(&report),
            [Fail, NotChecked, NotChecked, NotChecked, Pass]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.exit_code(), 3);
        assert!(report.items[1].text.ends_with("(jag has no key to send)"));
    }

    #[test]
    fn a_file_error_of_the_settings_shows_on_item_5_when_the_file_is_open() {
        let pairs = [("JAG_TYPESAFE_API_KEY", SENTINEL_KEY)];
        let (report, calls) = check(&pairs, true, &file(0o644, KEY_FILE_TEXT), Ok(()));
        assert_eq!(
            statuses(&report),
            [NotChecked, NotChecked, NotChecked, NotChecked, Fail]
        );
        assert_eq!(calls, 0);
        assert_eq!(report.failures().count(), 1);
        assert!(
            report.items[0]
                .text
                .ends_with("(jag cannot use the key file: see the last item)")
        );
    }

    fn item(status: Status, failure_code: Option<u8>) -> Item {
        Item {
            status,
            text: "x".to_owned(),
            failure: failure_code.map(|exit_code| Failure {
                cause: "c".to_owned(),
                fix: "f".to_owned(),
                exit_code,
            }),
        }
    }

    #[test]
    fn exit_code_is_0_3_4_5_6_3_for_no_failure_and_for_one_failed_item() {
        let codes = [3, 4, 5, 6, 3];
        let none = Report {
            items: std::array::from_fn(|_| item(Pass, None)),
        };
        assert_eq!(none.exit_code(), 0);
        for (index, code) in codes.into_iter().enumerate() {
            let report = Report {
                items: std::array::from_fn(|i| {
                    if i == index {
                        item(Fail, Some(code))
                    } else {
                        item(Pass, None)
                    }
                }),
            };
            assert_eq!(report.exit_code(), code, "item {}", index + 1);
        }
    }

    #[test]
    fn the_lines_have_a_status_field_of_12_characters() {
        let (report, _) = check(&[], true, &no_file(), Ok(()));
        assert_eq!(
            report.lines(),
            "fail        a key is present\n\
             not checked the service accepts the key (jag has no key to send)\n\
             not checked the account has credit (jag has no key to send)\n\
             not checked the service is reachable (jag has no key to send)\n\
             pass        other users cannot read the key file (no key file)\n"
        );
    }

    #[test]
    fn jag_is_ready_follows_five_passes_only() {
        let (report, _) = check(GOOD, true, &no_file(), Ok(()));
        assert_eq!(
            report.lines(),
            "pass        a key is present (JAG_TYPESAFE_API_KEY)\n\
             pass        the service accepts the key\n\
             pass        the account has credit\n\
             pass        the service is reachable\n\
             pass        other users cannot read the key file (no key file)\n\
             jag is ready.\n"
        );
        let (report, _) = check(GOOD, true, &no_file(), Err(ScoreError::NoCredit));
        assert!(!report.lines().contains("jag is ready."));
    }

    #[test]
    fn no_text_holds_the_key() {
        let (report, _) = check(
            GOOD,
            true,
            &file(0o644, KEY_FILE_TEXT),
            Err(ScoreError::NoCredit),
        );
        let mut text = report.lines();
        for failure in report.failures() {
            text.push_str(&failure.render());
        }
        assert!(!text.contains(SENTINEL_KEY));
    }

    #[test]
    fn the_real_file_system_gives_the_same_answer_for_a_missing_file() {
        let home = tempfile::tempdir().expect("temporary folder");
        let vars = vars(GOOD);
        let env = Env {
            vars: &vars,
            home: Some(home.path().to_owned()),
        };
        let file = ConfigFile::new(&env, &crate::config::RealFs);
        let settings =
            model_settings(&env, &file, &SettingOptions::default()).map(ScoreSettings::new);
        let report = run(&env, &file, settings, &|_, _| Ok(()));
        assert_eq!(statuses(&report), [Pass; 5]);
    }
}
