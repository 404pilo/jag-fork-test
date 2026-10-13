//! The settings of the model service: the address and the model name.
//!
//! No message here shows a value of `JAG_BASE_URL` or `JAG_MODEL`, because an
//! address can hold a user name and a password (CV-6).

use ureq::http::Uri;

use super::{ConfigError, Env};

pub const BASE_URL_VARIABLE: &str = "JAG_BASE_URL";
pub const MODEL_VARIABLE: &str = "JAG_MODEL";
pub const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai/v1";
pub const DEFAULT_MODEL: &str = "jev-1.13.0";

/// Where jag sends the calls, and which model it asks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSettings {
    /// With no `/` at the end. The call goes to `<base_url>/systemone`.
    pub base_url: String,
    pub model: String,
}

/// Read `JAG_BASE_URL` and `JAG_MODEL`. A blank variable counts as not set.
pub fn model_settings(env: &Env) -> Result<ModelSettings, ConfigError> {
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
    Ok(ModelSettings { base_url, model })
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
        let vars = vars(pairs);
        model_settings(&Env {
            vars: &vars,
            home: Some(PathBuf::from("/nowhere")),
        })
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
        let result = model_settings(&Env {
            vars: &vars,
            home: None,
        });
        assert_eq!(result, Err(ConfigError::ModelBad));
    }
}
