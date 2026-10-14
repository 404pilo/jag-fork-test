//! The settings of the model service: the address, the model name, and the
//! times of one call and of the retry.
//!
//! No message here shows a value of a base URL, a model name, or a time,
//! because an address can hold a user name and a password (CV-6).

use std::path::PathBuf;
use std::time::Duration;

use ureq::http::Uri;

use super::{ConfigError, ConfigFile, Env};

pub const BASE_URL_VARIABLE: &str = "JAG_BASE_URL";
pub const MODEL_VARIABLE: &str = "JAG_MODEL";
pub const BASE_URL_OPTION: &str = "--base-url";
pub const MODEL_OPTION: &str = "--model";
pub const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1";
pub const DEFAULT_MODEL: &str = "jev-1.13.0";
/// The price of `DEFAULT_MODEL` for one input token, in billionths of a US
/// dollar: 0.042 US dollars for 1 million tokens (FEATURE-20, decision 3).
pub const DEFAULT_MODEL_PRICE: u64 = 42;

/// The price of one input token of `model`, in billionths of a US dollar.
/// `None` for a model that jag has no price for (decision 3).
pub fn price_of(model: &str) -> Option<u64> {
    (model == DEFAULT_MODEL).then_some(DEFAULT_MODEL_PRICE)
}

pub const TIMEOUT_VARIABLE: &str = "JAG_TIMEOUT";
pub const RETRY_WAIT_VARIABLE: &str = "JAG_RETRY_WAIT";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(4);
pub const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(1);
pub const MIN_SECONDS: u64 = 1;
pub const MAX_SECONDS: u64 = 60;
/// The host of the TypeSafe API (decision 16).
const TYPESAFE_HOST: &str = "api.typesafe.ai";

/// Which of the two times a value is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSetting {
    Timeout,
    RetryWait,
}

/// Where a value of a setting came from, for a setting that has no file key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingSource {
    Option,
    Variable,
}

/// Where the calls go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    /// The TypeSafe API (IF-1).
    #[default]
    Direct,
    /// An AI gateway (IF-2).
    Gateway,
}

/// Where the value of the base URL or of the model came from (CV-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    Option,
    Variable,
    ConfigFile(PathBuf),
    #[default]
    Default,
}

impl Origin {
    /// The name of the source of the base URL, for a fix. `None` for the
    /// default.
    pub fn base_url_name(&self) -> Option<String> {
        self.name(BASE_URL_OPTION, BASE_URL_VARIABLE, "config.base-url")
    }

    /// The name of the source of the model, for a fix. `None` for the default.
    pub fn model_name(&self) -> Option<String> {
        self.name(MODEL_OPTION, MODEL_VARIABLE, "config.model")
    }

    fn name(&self, option: &str, variable: &str, key: &str) -> Option<String> {
        match self {
            Origin::Option => Some(option.to_owned()),
            Origin::Variable => Some(variable.to_owned()),
            Origin::ConfigFile(path) => Some(format!("{key} in {}", path.display())),
            Origin::Default => None,
        }
    }
}

/// The time of one call, and the wait before the one retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Times {
    pub timeout: Duration,
    pub retry_wait: Duration,
}

impl Default for Times {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            retry_wait: DEFAULT_RETRY_WAIT,
        }
    }
}

/// A whole number of seconds from 1 to 60: 1 to 3 ASCII digits and nothing
/// else. The error never holds the text.
pub fn parse_seconds(
    text: &str,
    setting: TimeSetting,
    source: SettingSource,
) -> Result<Duration, ConfigError> {
    let text = text.trim();
    let bad = ConfigError::TimeBad { setting, source };
    if text.is_empty() || text.len() > 3 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(bad);
    }
    match text.parse::<u64>() {
        Ok(seconds) if (MIN_SECONDS..=MAX_SECONDS).contains(&seconds) => {
            Ok(Duration::from_secs(seconds))
        }
        _ => Err(bad),
    }
}

/// The options of step 1, after their checks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingOptions {
    /// After the address rule.
    pub base_url: Option<String>,
    /// Trimmed. Not blank.
    pub model: Option<String>,
    /// `(timeout, retry wait)`.
    pub times: (Option<Duration>, Option<Duration>),
}

/// Where jag sends the calls, and which model it asks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSettings {
    /// After the address rule, so it ends with `/v1`. The call goes to
    /// `<base_url>/systemone`.
    pub base_url: String,
    pub base_url_origin: Origin,
    pub model: String,
    pub model_origin: Origin,
    pub route: Route,
    pub times: Times,
}

/// The address rule (decisions 7 and 8). `None` when jag refuses the address.
pub fn normalize_base_url(text: &str) -> Option<String> {
    let text = text.trim().trim_end_matches('/');
    if text.contains(['?', '#']) || !address_is_allowed(text) {
        return None;
    }
    let uri = text.parse::<Uri>().ok()?;
    if uri.path().trim_end_matches('/').ends_with("/v1") {
        Some(text.to_owned())
    } else {
        Some(format!("{text}/v1"))
    }
}

/// The route of a base URL that the address rule gave (decision 16).
pub fn route_of(base_url: &str) -> Route {
    let Ok(uri) = base_url.parse::<Uri>() else {
        return Route::Gateway;
    };
    let direct = uri.scheme_str() == Some("https")
        && uri
            .host()
            .is_some_and(|host| host.eq_ignore_ascii_case(TYPESAFE_HOST))
        && uri.port_u16().is_none_or(|port| port == 443)
        && uri.path() == "/v1";
    if direct {
        Route::Direct
    } else {
        Route::Gateway
    }
}

/// Find the base URL, the model, the timeout, and the retry wait. For the
/// base URL and the model, the first source that is set wins: the option, the
/// variable, the config file, the default (decision 5). jag reads the file
/// only when a setting needs it (decision 12). A blank variable, or a blank
/// value in the file, counts as not set.
pub fn model_settings(
    env: &Env,
    file: &ConfigFile,
    options: &SettingOptions,
) -> Result<ModelSettings, ConfigError> {
    let (base_url, base_url_origin) = base_url(env, file, options)?;
    let (model, model_origin) = model(env, file, options)?;
    let timeout = time(
        env,
        options.times.0,
        TIMEOUT_VARIABLE,
        TimeSetting::Timeout,
        DEFAULT_TIMEOUT,
    )?;
    let retry_wait = time(
        env,
        options.times.1,
        RETRY_WAIT_VARIABLE,
        TimeSetting::RetryWait,
        DEFAULT_RETRY_WAIT,
    )?;
    let route = route_of(&base_url);
    Ok(ModelSettings {
        base_url,
        base_url_origin,
        model,
        model_origin,
        route,
        times: Times {
            timeout,
            retry_wait,
        },
    })
}

fn base_url(
    env: &Env,
    file: &ConfigFile,
    options: &SettingOptions,
) -> Result<(String, Origin), ConfigError> {
    if let Some(url) = &options.base_url {
        return Ok((url.clone(), Origin::Option));
    }
    let bad = ConfigError::BaseUrlBad {
        source: SettingSource::Variable,
    };
    if let Some(text) = variable(env, BASE_URL_VARIABLE, &bad)? {
        return normalize_base_url(&text)
            .map(|url| (url, Origin::Variable))
            .ok_or(bad);
    }
    if let Some((path, parsed)) = file.get()?
        && let Some(text) = parsed
            .config
            .as_ref()
            .and_then(|section| section.base_url.as_deref())
            .map(str::trim)
            .filter(|text| !text.is_empty())
    {
        return normalize_base_url(text)
            .map(|url| (url, Origin::ConfigFile(path.to_owned())))
            .ok_or_else(|| ConfigError::FileBaseUrlBad {
                path: path.to_owned(),
            });
    }
    Ok((DEFAULT_BASE_URL.to_owned(), Origin::Default))
}

fn model(
    env: &Env,
    file: &ConfigFile,
    options: &SettingOptions,
) -> Result<(String, Origin), ConfigError> {
    if let Some(name) = &options.model {
        return Ok((name.clone(), Origin::Option));
    }
    let bad = ConfigError::ModelBad {
        source: SettingSource::Variable,
    };
    if let Some(text) = variable(env, MODEL_VARIABLE, &bad)? {
        return Ok((text, Origin::Variable));
    }
    if let Some((path, parsed)) = file.get()?
        && let Some(text) = parsed
            .config
            .as_ref()
            .and_then(|section| section.model.as_deref())
            .map(str::trim)
            .filter(|text| !text.is_empty())
    {
        return Ok((text.to_owned(), Origin::ConfigFile(path.to_owned())));
    }
    Ok((DEFAULT_MODEL.to_owned(), Origin::Default))
}

/// The option, else the variable, else the default.
fn time(
    env: &Env,
    option: Option<Duration>,
    name: &str,
    setting: TimeSetting,
    default: Duration,
) -> Result<Duration, ConfigError> {
    if let Some(option) = option {
        return Ok(option);
    }
    match (env.vars)(name) {
        None => Ok(default),
        Some(value) => {
            let bad = ConfigError::TimeBad {
                setting,
                source: SettingSource::Variable,
            };
            let text = value.to_str().ok_or(bad)?;
            if text.trim().is_empty() {
                return Ok(default);
            }
            parse_seconds(text, setting, SettingSource::Variable)
        }
    }
}

/// The trimmed text of a variable, or `None` when it is not set or blank. A
/// value that is not text gives `bad`.
fn variable(env: &Env, name: &str, bad: &ConfigError) -> Result<Option<String>, ConfigError> {
    let Some(value) = (env.vars)(name) else {
        return Ok(None);
    };
    let text = value.to_str().ok_or_else(|| bad.clone())?;
    let text = text.trim();
    Ok((!text.is_empty()).then(|| text.to_owned()))
}

/// `https`, or `http` for a host on this machine. No user name, no password
/// (decision 29). The whole host is compared, not a part of it.
fn address_is_allowed(text: &str) -> bool {
    let Ok(uri) = text.parse::<Uri>() else {
        return false;
    };
    let Some(authority) = uri.authority() else {
        return false;
    };
    if authority.as_str().contains('@') {
        return false;
    }
    match uri.scheme_str() {
        Some("https") => true,
        Some("http") => matches!(authority.host(), "127.0.0.1" | "localhost" | "[::1]"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_pinned_model_has_a_price_and_other_names_have_none() {
        assert_eq!(price_of(DEFAULT_MODEL), Some(42));
        assert_eq!(price_of("jev-1.13"), None);
        assert_eq!(price_of(""), None);
    }

    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::path::Path;

    use super::super::RealFs;
    use super::super::testing::{vars, write_file};
    use super::*;

    fn run(
        pairs: &[(&str, &str)],
        home: Option<&Path>,
        options: &SettingOptions,
    ) -> Result<ModelSettings, ConfigError> {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        let file = ConfigFile::new(&env, &RealFs);
        model_settings(&env, &file, options)
    }

    fn settings(pairs: &[(&str, &str)]) -> Result<ModelSettings, ConfigError> {
        run(
            pairs,
            Some(Path::new("/nowhere")),
            &SettingOptions::default(),
        )
    }

    fn with_options(
        pairs: &[(&str, &str)],
        options: (Option<Duration>, Option<Duration>),
    ) -> Result<ModelSettings, ConfigError> {
        let options = SettingOptions {
            times: options,
            ..SettingOptions::default()
        };
        run(pairs, Some(Path::new("/nowhere")), &options)
    }

    /// A home with a config file of this text and mode 0600.
    fn home_with(text: &str) -> tempfile::TempDir {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), text, 0o600);
        home
    }

    #[test]
    fn the_defaults_are_the_typesafe_address_and_the_pinned_model() {
        let settings = settings(&[]).expect("defaults");
        assert_eq!(settings.base_url, "https://api.typesafe.ai/v1");
        assert_eq!(settings.model, "jev-1.13.0");
        assert_eq!(settings.base_url_origin, Origin::Default);
        assert_eq!(settings.model_origin, Origin::Default);
        assert_eq!(settings.route, Route::Direct);
    }

    #[test]
    fn a_blank_variable_counts_as_not_set() {
        let settings =
            settings(&[(BASE_URL_VARIABLE, "  "), (MODEL_VARIABLE, "\t")]).expect("defaults");
        assert_eq!(settings.base_url, DEFAULT_BASE_URL);
        assert_eq!(settings.model, DEFAULT_MODEL);
        assert_eq!(settings.base_url_origin, Origin::Default);
    }

    #[test]
    fn the_variables_are_trimmed_and_the_slash_at_the_end_is_removed() {
        let settings = settings(&[
            (BASE_URL_VARIABLE, " https://gateway.example.net/v1// "),
            (MODEL_VARIABLE, " jev-latest "),
        ])
        .expect("settings");
        assert_eq!(settings.base_url, "https://gateway.example.net/v1");
        assert_eq!(settings.model, "jev-latest");
        assert_eq!(settings.base_url_origin, Origin::Variable);
        assert_eq!(settings.model_origin, Origin::Variable);
    }

    // The table of "The address rule".
    #[test]
    fn the_address_rule_gives_the_base_url_of_each_form() {
        for (text, want) in [
            ("https://api.typesafe.ai", "https://api.typesafe.ai/v1"),
            ("https://api.typesafe.ai/", "https://api.typesafe.ai/v1"),
            ("https://api.typesafe.ai/v1", "https://api.typesafe.ai/v1"),
            ("https://openrouter.ai/api", "https://openrouter.ai/api/v1"),
            (
                "https://ai-gateway.vercel.sh/typesafe",
                "https://ai-gateway.vercel.sh/typesafe/v1",
            ),
            (
                "https://gateway-us.pydantic.dev/proxy/typesafe",
                "https://gateway-us.pydantic.dev/proxy/typesafe/v1",
            ),
            (
                "https://gateway-us.pydantic.dev/proxy/typesafe/",
                "https://gateway-us.pydantic.dev/proxy/typesafe/v1",
            ),
            ("http://127.0.0.1:8080/v1", "http://127.0.0.1:8080/v1"),
            ("http://127.0.0.1:8080", "http://127.0.0.1:8080/v1"),
            (
                "https://example.com/v1beta",
                "https://example.com/v1beta/v1",
            ),
            ("https://example.com/V1", "https://example.com/V1/v1"),
            (
                "  https://example.com/api/v1/  ",
                "https://example.com/api/v1",
            ),
        ] {
            assert_eq!(normalize_base_url(text).as_deref(), Some(want), "{text}");
        }
    }

    #[test]
    fn the_address_rule_refuses_these_texts() {
        for bad in [
            "https://example.com/v1?a=1",
            "https://example.com/#a",
            "https://user@example.com",
            "https://user:pass@example.com/v1",
            "http://example.com",
            "http://localhost.example.net/v1",
            "http://127.0.0.1@example.net/v1",
            "ftp://example.com",
            "ftp://127.0.0.1/v1",
            "example.com",
            "example.com/v1",
            "not an address",
            "/v1",
            "",
            "   ",
        ] {
            assert_eq!(normalize_base_url(bad), None, "{bad:?}");
        }
        for good in [
            "http://127.0.0.1:8080/v1",
            "http://localhost:8080/v1",
            "http://[::1]:8080/v1",
            "https://example.com/v1",
        ] {
            assert!(normalize_base_url(good).is_some(), "{good}");
        }
    }

    #[test]
    fn each_source_refuses_a_bad_address_with_its_own_error() {
        for bad in [
            "http://example.com",
            "https://a@example.com",
            "ftp://x",
            "a?b",
        ] {
            let variable = settings(&[(BASE_URL_VARIABLE, bad)]);
            assert_eq!(
                variable,
                Err(ConfigError::BaseUrlBad {
                    source: SettingSource::Variable
                }),
                "{bad}"
            );
            let home = home_with(&format!("config:\n  base-url: \"{bad}\"\n"));
            let file = run(&[], Some(home.path()), &SettingOptions::default());
            let path = home.path().join(".config/jag/config.yml");
            assert_eq!(file, Err(ConfigError::FileBaseUrlBad { path }), "{bad}");
        }
    }

    #[test]
    fn the_route_is_direct_only_for_the_typesafe_address() {
        for direct in [
            "https://api.typesafe.ai/v1",
            "https://API.TypeSafe.ai/v1",
            "https://api.typesafe.ai:443/v1",
        ] {
            assert_eq!(route_of(direct), Route::Direct, "{direct}");
        }
        for gateway in [
            "https://api.typesafe.ai:8443/v1",
            "https://openrouter.ai/api/v1",
            "http://127.0.0.1:8080/v1",
            "http://api.typesafe.ai/v1",
            "https://api.typesafe.ai/other/v1",
            "https://api.typesafe.ai.example.com/v1",
        ] {
            assert_eq!(route_of(gateway), Route::Gateway, "{gateway}");
        }
        let direct = settings(&[(BASE_URL_VARIABLE, "https://api.typesafe.ai")]).expect("direct");
        assert_eq!(direct.route, Route::Direct);
        let gateway = settings(&[(BASE_URL_VARIABLE, "https://openrouter.ai/api")]).expect("gw");
        assert_eq!(gateway.route, Route::Gateway);
    }

    #[test]
    fn the_host_of_an_ipv6_address_has_square_brackets() {
        let uri: Uri = "http://[::1]:8080/v1".parse().expect("an address");
        assert_eq!(uri.host(), Some("[::1]"));
        let authority = uri.authority().expect("an authority");
        assert_eq!(authority.host(), "[::1]");
    }

    #[test]
    fn a_variable_that_is_not_text_is_refused() {
        for (name, want) in [
            (
                MODEL_VARIABLE,
                ConfigError::ModelBad {
                    source: SettingSource::Variable,
                },
            ),
            (
                BASE_URL_VARIABLE,
                ConfigError::BaseUrlBad {
                    source: SettingSource::Variable,
                },
            ),
        ] {
            let vars = |var: &str| (var == name).then(|| OsString::from_vec(vec![0xff]));
            let env = Env {
                vars: &vars,
                home: None,
            };
            let file = ConfigFile::new(&env, &RealFs);
            let result = model_settings(&env, &file, &SettingOptions::default());
            assert_eq!(result, Err(want));
        }
    }

    // CRITERION-96: each pair of sources, for each setting.
    #[test]
    fn the_option_wins_over_the_variable_and_the_variable_over_the_file() {
        let home =
            home_with("config:\n  base-url: \"https://file.example.com/api\"\n  model: m-file\n");
        let file_path = home.path().join(".config/jag/config.yml");
        let option = SettingOptions {
            base_url: Some("https://option.example.com/v1".to_owned()),
            model: Some("m-option".to_owned()),
            ..SettingOptions::default()
        };
        let pairs = [
            (BASE_URL_VARIABLE, "https://variable.example.com"),
            (MODEL_VARIABLE, "m-variable"),
        ];
        let all = run(&pairs, Some(home.path()), &option).expect("option");
        assert_eq!(all.base_url, "https://option.example.com/v1");
        assert_eq!(all.model, "m-option");
        assert_eq!(all.base_url_origin, Origin::Option);
        assert_eq!(all.model_origin, Origin::Option);

        let variable = run(&pairs, Some(home.path()), &SettingOptions::default()).expect("var");
        assert_eq!(variable.base_url, "https://variable.example.com/v1");
        assert_eq!(variable.model, "m-variable");
        assert_eq!(variable.base_url_origin, Origin::Variable);
        assert_eq!(variable.model_origin, Origin::Variable);

        let file = run(&[], Some(home.path()), &SettingOptions::default()).expect("file");
        assert_eq!(file.base_url, "https://file.example.com/api/v1");
        assert_eq!(file.model, "m-file");
        assert_eq!(file.base_url_origin, Origin::ConfigFile(file_path.clone()));
        assert_eq!(file.model_origin, Origin::ConfigFile(file_path));
    }

    #[test]
    fn each_setting_has_its_own_order() {
        let home = home_with("config:\n  model: m-file\n");
        let option = SettingOptions {
            base_url: Some("https://option.example.com/v1".to_owned()),
            ..SettingOptions::default()
        };
        let settings = run(&[], Some(home.path()), &option).expect("settings");
        assert_eq!(settings.base_url_origin, Origin::Option);
        assert_eq!(settings.model, "m-file");
        assert!(matches!(settings.model_origin, Origin::ConfigFile(_)));
        assert_eq!(
            settings.base_url_origin.base_url_name().as_deref(),
            Some("--base-url")
        );
    }

    #[test]
    fn a_source_that_wins_hides_a_bad_lower_value() {
        let home = home_with("config:\n  base-url: \"ftp://example.com\"\n");
        let option = SettingOptions {
            base_url: Some("https://option.example.com/v1".to_owned()),
            ..SettingOptions::default()
        };
        let pairs = [(BASE_URL_VARIABLE, "http://example.com")];
        assert!(run(&pairs, Some(home.path()), &option).is_ok());
        let pairs = [(BASE_URL_VARIABLE, "https://variable.example.com")];
        assert!(run(&pairs, Some(home.path()), &SettingOptions::default()).is_ok());
    }

    #[test]
    fn a_blank_value_in_the_file_counts_as_not_set() {
        let home = home_with("config:\n  base-url: \"  \"\n  model: \"\"\n");
        let settings = run(&[], Some(home.path()), &SettingOptions::default()).expect("settings");
        assert_eq!(settings.base_url_origin, Origin::Default);
        assert_eq!(settings.model_origin, Origin::Default);
    }

    #[test]
    fn jag_reads_the_file_only_when_a_setting_needs_it() {
        // A file that others can read gives an error only when it is read.
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), "config:\n  model: m\n", 0o644);
        let both = [
            (BASE_URL_VARIABLE, "https://gw.example.com"),
            (MODEL_VARIABLE, "m"),
        ];
        assert!(run(&both, Some(home.path()), &SettingOptions::default()).is_ok());
        let one = [(BASE_URL_VARIABLE, "https://gw.example.com")];
        assert!(matches!(
            run(&one, Some(home.path()), &SettingOptions::default()),
            Err(ConfigError::KeyFileOpen { .. })
        ));
    }

    #[test]
    fn the_names_of_the_sources_are_for_the_fixes() {
        let path = std::path::PathBuf::from("/h/config.yml");
        assert_eq!(Origin::Default.base_url_name(), None);
        assert_eq!(Origin::Variable.model_name().as_deref(), Some("JAG_MODEL"));
        assert_eq!(
            Origin::ConfigFile(path.clone()).base_url_name().as_deref(),
            Some("config.base-url in /h/config.yml")
        );
        assert_eq!(
            Origin::ConfigFile(path).model_name().as_deref(),
            Some("config.model in /h/config.yml")
        );
    }

    const BOTH: [TimeSetting; 2] = [TimeSetting::Timeout, TimeSetting::RetryWait];

    #[test]
    fn whole_seconds_from_1_to_60_pass() {
        for (text, seconds) in [("1", 1), ("4", 4), ("60", 60), ("01", 1), (" 10 ", 10)] {
            for setting in BOTH {
                assert_eq!(
                    parse_seconds(text, setting, SettingSource::Option),
                    Ok(Duration::from_secs(seconds)),
                    "should accept {text:?}"
                );
            }
        }
    }

    #[test]
    fn other_values_are_refused_for_both_settings_and_both_sources() {
        for text in [
            "0", "61", "0.5", "1.5", "-1", "+1", "1e1", "4s", "1000", "abc", "", "  ",
        ] {
            for setting in BOTH {
                for source in [SettingSource::Option, SettingSource::Variable] {
                    assert_eq!(
                        parse_seconds(text, setting, source),
                        Err(ConfigError::TimeBad { setting, source }),
                        "should refuse {text:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_times_default_to_4_and_1_seconds() {
        let settings = settings(&[]).expect("defaults");
        assert_eq!(settings.times, Times::default());
        assert_eq!(settings.times.timeout, Duration::from_secs(4));
        assert_eq!(settings.times.retry_wait, Duration::from_secs(1));
    }

    #[test]
    fn a_time_variable_is_used_and_a_blank_one_is_not_set() {
        let set = settings(&[(TIMEOUT_VARIABLE, "7"), (RETRY_WAIT_VARIABLE, " 3 ")]).expect("set");
        assert_eq!(set.times.timeout, Duration::from_secs(7));
        assert_eq!(set.times.retry_wait, Duration::from_secs(3));
        let blank = settings(&[(TIMEOUT_VARIABLE, " "), (RETRY_WAIT_VARIABLE, "")]).expect("blank");
        assert_eq!(blank.times, Times::default());
    }

    #[test]
    fn a_time_option_wins_over_its_variable() {
        let settings = with_options(
            &[(TIMEOUT_VARIABLE, "30"), (RETRY_WAIT_VARIABLE, "3")],
            (Some(Duration::from_secs(1)), Some(Duration::from_secs(2))),
        )
        .expect("settings");
        assert_eq!(settings.times.timeout, Duration::from_secs(1));
        assert_eq!(settings.times.retry_wait, Duration::from_secs(2));
    }

    #[test]
    fn an_option_hides_a_bad_variable_of_the_same_time() {
        let settings = with_options(
            &[(TIMEOUT_VARIABLE, "abc")],
            (Some(Duration::from_secs(5)), None),
        )
        .expect("settings");
        assert_eq!(settings.times.timeout, Duration::from_secs(5));
    }

    #[test]
    fn a_bad_time_variable_is_refused_and_the_timeout_is_checked_first() {
        assert_eq!(
            settings(&[(TIMEOUT_VARIABLE, "abc")]),
            Err(ConfigError::TimeBad {
                setting: TimeSetting::Timeout,
                source: SettingSource::Variable
            })
        );
        assert_eq!(
            settings(&[(RETRY_WAIT_VARIABLE, "61")]),
            Err(ConfigError::TimeBad {
                setting: TimeSetting::RetryWait,
                source: SettingSource::Variable
            })
        );
        assert_eq!(
            settings(&[(TIMEOUT_VARIABLE, "0"), (RETRY_WAIT_VARIABLE, "0")]),
            Err(ConfigError::TimeBad {
                setting: TimeSetting::Timeout,
                source: SettingSource::Variable
            })
        );
    }

    #[test]
    fn a_time_variable_that_is_not_text_is_refused() {
        let vars =
            |name: &str| (name == RETRY_WAIT_VARIABLE).then(|| OsString::from_vec(vec![0xff]));
        let env = Env {
            vars: &vars,
            home: None,
        };
        let file = ConfigFile::new(&env, &RealFs);
        let result = model_settings(&env, &file, &SettingOptions::default());
        assert_eq!(
            result,
            Err(ConfigError::TimeBad {
                setting: TimeSetting::RetryWait,
                source: SettingSource::Variable
            })
        );
    }
}
