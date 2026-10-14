//! The settings of the model service: the address, the model name, and the
//! times of one call and of the retry.
//!
//! No message here shows a value of `JAG_BASE_URL`, `JAG_MODEL`,
//! `JAG_TIMEOUT`, or `JAG_RETRY_WAIT`, because an address can hold a user
//! name and a password (CV-6).

use std::time::Duration;

use ureq::http::Uri;

use super::{ConfigError, Env};

pub const BASE_URL_VARIABLE: &str = "JAG_BASE_URL";
pub const MODEL_VARIABLE: &str = "JAG_MODEL";
pub const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1";
pub const DEFAULT_MODEL: &str = "jev-1.13.0";
pub const TIMEOUT_VARIABLE: &str = "JAG_TIMEOUT";
pub const RETRY_WAIT_VARIABLE: &str = "JAG_RETRY_WAIT";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(4);
pub const DEFAULT_RETRY_WAIT: Duration = Duration::from_secs(1);
pub const MIN_SECONDS: u64 = 1;
pub const MAX_SECONDS: u64 = 60;

/// Which of the two times a value is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSetting {
    Timeout,
    RetryWait,
}

/// Where a time value came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSource {
    Option,
    Variable,
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
    source: TimeSource,
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

/// Where jag sends the calls, and which model it asks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSettings {
    /// With no `/` at the end. The call goes to `<base_url>/systemone`.
    pub base_url: String,
    pub model: String,
    pub times: Times,
}

/// Read `JAG_BASE_URL`, `JAG_MODEL`, `JAG_TIMEOUT`, and `JAG_RETRY_WAIT`. A
/// blank variable counts as not set. A time option, already parsed, wins
/// over its variable (decision 9). `options` is `(timeout, retry wait)`.
pub fn model_settings(
    env: &Env,
    options: (Option<Duration>, Option<Duration>),
) -> Result<ModelSettings, ConfigError> {
    let base_url = match variable(env, BASE_URL_VARIABLE)? {
        Some(text) => {
            let text = text.trim_end_matches('/').to_owned();
            if !address_is_allowed(&text) {
                return Err(ConfigError::BaseUrlBad);
            }
            text
        }
        None => DEFAULT_BASE_URL.to_owned(),
    };
    let model = variable(env, MODEL_VARIABLE)?.unwrap_or_else(|| DEFAULT_MODEL.to_owned());
    let timeout = time(
        env,
        options.0,
        TIMEOUT_VARIABLE,
        TimeSetting::Timeout,
        DEFAULT_TIMEOUT,
    )?;
    let retry_wait = time(
        env,
        options.1,
        RETRY_WAIT_VARIABLE,
        TimeSetting::RetryWait,
        DEFAULT_RETRY_WAIT,
    )?;
    Ok(ModelSettings {
        base_url,
        model,
        times: Times {
            timeout,
            retry_wait,
        },
    })
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
                source: TimeSource::Variable,
            };
            let text = value.to_str().ok_or(bad)?;
            if text.trim().is_empty() {
                return Ok(default);
            }
            parse_seconds(text, setting, TimeSource::Variable)
        }
    }
}

/// The trimmed text of a variable, or `None` when it is not set or blank.
fn variable(env: &Env, name: &str) -> Result<Option<String>, ConfigError> {
    let Some(value) = (env.vars)(name) else {
        return Ok(None);
    };
    let text = value.to_str().ok_or(if name == BASE_URL_VARIABLE {
        ConfigError::BaseUrlBad
    } else {
        ConfigError::ModelBad
    })?;
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
    use std::path::PathBuf;

    use super::super::testing::vars;
    use super::*;

    fn settings(pairs: &[(&str, &str)]) -> Result<ModelSettings, ConfigError> {
        with_options(pairs, (None, None))
    }

    fn with_options(
        pairs: &[(&str, &str)],
        options: (Option<Duration>, Option<Duration>),
    ) -> Result<ModelSettings, ConfigError> {
        let vars = vars(pairs);
        model_settings(
            &Env {
                vars: &vars,
                home: Some(PathBuf::from("/nowhere")),
            },
            options,
        )
    }

    #[test]
    fn the_defaults_are_the_typesafe_address_and_the_pinned_model() {
        let settings = settings(&[]).expect("defaults");
        assert_eq!(settings.base_url, "https://api.typesafe.ai/v1");
        assert_eq!(settings.model, "jev-1.13.0");
    }

    #[test]
    fn a_blank_variable_counts_as_not_set() {
        let settings =
            settings(&[(BASE_URL_VARIABLE, "  "), (MODEL_VARIABLE, "\t")]).expect("defaults");
        assert_eq!(settings.base_url, DEFAULT_BASE_URL);
        assert_eq!(settings.model, DEFAULT_MODEL);
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
    }

    #[test]
    fn plain_http_is_allowed_only_for_this_machine() {
        for good in [
            "http://127.0.0.1:8080/v1",
            "http://localhost:8080/v1",
            "http://[::1]:8080/v1",
            "https://example.com/v1",
        ] {
            assert!(
                settings(&[(BASE_URL_VARIABLE, good)]).is_ok(),
                "should accept {good}"
            );
        }
        for bad in [
            "http://example.com/v1",
            "http://localhost.example.net/v1",
            "http://127.0.0.1@example.net/v1",
            "https://user@example.com/v1",
            "https://user:pass@example.com/v1",
            "ftp://127.0.0.1/v1",
            "example.com/v1",
            "not an address",
            "/v1",
        ] {
            assert_eq!(
                settings(&[(BASE_URL_VARIABLE, bad)]),
                Err(ConfigError::BaseUrlBad),
                "should refuse {bad}"
            );
        }
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
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let vars = |name: &str| (name == MODEL_VARIABLE).then(|| OsString::from_vec(vec![0xff]));
        let result = model_settings(
            &Env {
                vars: &vars,
                home: None,
            },
            (None, None),
        );
        assert_eq!(result, Err(ConfigError::ModelBad));
    }

    const BOTH: [TimeSetting; 2] = [TimeSetting::Timeout, TimeSetting::RetryWait];

    #[test]
    fn whole_seconds_from_1_to_60_pass() {
        for (text, seconds) in [("1", 1), ("4", 4), ("60", 60), ("01", 1), (" 10 ", 10)] {
            for setting in BOTH {
                assert_eq!(
                    parse_seconds(text, setting, TimeSource::Option),
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
                for source in [TimeSource::Option, TimeSource::Variable] {
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
                source: TimeSource::Variable
            })
        );
        assert_eq!(
            settings(&[(RETRY_WAIT_VARIABLE, "61")]),
            Err(ConfigError::TimeBad {
                setting: TimeSetting::RetryWait,
                source: TimeSource::Variable
            })
        );
        assert_eq!(
            settings(&[(TIMEOUT_VARIABLE, "0"), (RETRY_WAIT_VARIABLE, "0")]),
            Err(ConfigError::TimeBad {
                setting: TimeSetting::Timeout,
                source: TimeSource::Variable
            })
        );
    }

    #[test]
    fn a_time_variable_that_is_not_text_is_refused() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let vars =
            |name: &str| (name == RETRY_WAIT_VARIABLE).then(|| OsString::from_vec(vec![0xff]));
        let result = model_settings(
            &Env {
                vars: &vars,
                home: None,
            },
            (None, None),
        );
        assert_eq!(
            result,
            Err(ConfigError::TimeBad {
                setting: TimeSetting::RetryWait,
                source: TimeSource::Variable
            })
        );
    }
}
