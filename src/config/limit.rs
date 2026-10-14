//! The cost limit: the most results that the model scores (STORY-19).
//!
//! No message here shows a value of `--max-scored` or `JAG_MAX_SCORED`
//! (CV-6).

use super::model::SettingSource;
use super::{ConfigError, Env};

pub const COST_LIMIT_OPTION: &str = "--max-scored";
pub const COST_LIMIT_VARIABLE: &str = "JAG_MAX_SCORED";
pub const DEFAULT_COST_LIMIT: usize = 200;
pub const MAX_COST_LIMIT: usize = 2_000;

/// A whole number from 1 to 2,000: 1 to 4 ASCII digits and nothing else. The
/// error never holds the text.
pub fn parse_cost_limit(text: &str, source: SettingSource) -> Result<usize, ConfigError> {
    let text = text.trim();
    let bad = ConfigError::CostLimitBad { source };
    if text.is_empty() || text.len() > 4 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(bad);
    }
    match text.parse::<usize>() {
        Ok(count) if (1..=MAX_COST_LIMIT).contains(&count) => Ok(count),
        _ => Err(bad),
    }
}

/// The option, else `JAG_MAX_SCORED`, else the default. A blank variable
/// counts as not set.
pub fn cost_limit(env: &Env, option: Option<usize>) -> Result<usize, ConfigError> {
    if let Some(option) = option {
        return Ok(option);
    }
    match (env.vars)(COST_LIMIT_VARIABLE) {
        None => Ok(DEFAULT_COST_LIMIT),
        Some(value) => {
            let bad = ConfigError::CostLimitBad {
                source: SettingSource::Variable,
            };
            let text = value.to_str().ok_or(bad)?;
            if text.trim().is_empty() {
                return Ok(DEFAULT_COST_LIMIT);
            }
            parse_cost_limit(text, SettingSource::Variable)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::testing::vars;
    use super::*;

    const SOURCES: [SettingSource; 2] = [SettingSource::Option, SettingSource::Variable];

    fn limit(pairs: &[(&str, &str)], option: Option<usize>) -> Result<usize, ConfigError> {
        let vars = vars(pairs);
        cost_limit(
            &Env {
                vars: &vars,
                home: Some(PathBuf::from("/nowhere")),
            },
            option,
        )
    }

    #[test]
    fn whole_numbers_from_1_to_2000_pass() {
        for (text, count) in [
            ("1", 1),
            ("200", 200),
            ("2000", 2000),
            ("0400", 400),
            (" 10 ", 10),
        ] {
            for source in SOURCES {
                assert_eq!(parse_cost_limit(text, source), Ok(count), "{text:?}");
            }
        }
    }

    #[test]
    fn other_values_are_refused_for_both_sources() {
        for text in [
            "0",
            "2001",
            "02000",
            "1.5",
            "-1",
            "+1",
            "1e3",
            "2,000",
            "200 results",
            "abc",
            "",
            "  ",
        ] {
            for source in SOURCES {
                assert_eq!(
                    parse_cost_limit(text, source),
                    Err(ConfigError::CostLimitBad { source }),
                    "{text:?}"
                );
            }
        }
    }

    #[test]
    fn the_default_is_200() {
        assert_eq!(limit(&[], None), Ok(DEFAULT_COST_LIMIT));
        assert_eq!(DEFAULT_COST_LIMIT, 200);
    }

    #[test]
    fn the_option_wins_over_the_variable() {
        assert_eq!(limit(&[(COST_LIMIT_VARIABLE, "20")], Some(10)), Ok(10));
        // An option hides a bad variable.
        assert_eq!(limit(&[(COST_LIMIT_VARIABLE, "abc")], Some(10)), Ok(10));
    }

    #[test]
    fn the_variable_is_used_and_a_blank_one_is_not_set() {
        assert_eq!(limit(&[(COST_LIMIT_VARIABLE, " 20 ")], None), Ok(20));
        assert_eq!(limit(&[(COST_LIMIT_VARIABLE, " ")], None), Ok(200));
    }

    #[test]
    fn a_bad_variable_is_refused() {
        assert_eq!(
            limit(&[(COST_LIMIT_VARIABLE, "2001")], None),
            Err(ConfigError::CostLimitBad {
                source: SettingSource::Variable
            })
        );
    }

    #[test]
    fn a_variable_that_is_not_text_is_refused() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let vars = |_: &str| Some(OsString::from_vec(vec![0xff]));
        let result = cost_limit(
            &Env {
                vars: &vars,
                home: None,
            },
            None,
        );
        assert_eq!(
            result,
            Err(ConfigError::CostLimitBad {
                source: SettingSource::Variable
            })
        );
    }
}
