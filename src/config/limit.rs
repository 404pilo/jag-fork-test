//! The cost limit: the most results that the model scores (STORY-19), and the
//! size limit: the most characters that jag writes to standard output
//! (STORY-25, STORY-28).
//!
//! No message here shows a value of `--max-scored`, `JAG_MAX_SCORED`,
//! `--max-chars`, or `JAG_MAX_CHARS` (CV-6).

use super::model::SettingSource;
use super::{ConfigError, Env};

pub const COST_LIMIT_OPTION: &str = "--max-scored";
pub const COST_LIMIT_VARIABLE: &str = "JAG_MAX_SCORED";
pub const DEFAULT_COST_LIMIT: usize = 200;
pub const MAX_COST_LIMIT: usize = 2_000;

pub const SIZE_LIMIT_OPTION: &str = "--max-chars";
pub const SIZE_LIMIT_OFF_OPTION: &str = "--no-max-chars";
pub const SIZE_LIMIT_VARIABLE: &str = "JAG_MAX_CHARS";
pub const DEFAULT_SIZE_LIMIT: usize = 20_000;
pub const MIN_SIZE_LIMIT: usize = 1_000;

/// The size limit of a search (CAPABILITY-04).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeLimit {
    /// The most characters on standard output.
    Chars(usize),
    /// `--no-max-chars`: no limit.
    Off,
}

impl SizeLimit {
    /// The count of characters, or `None` for no limit.
    pub fn chars(self) -> Option<usize> {
        match self {
            SizeLimit::Chars(count) => Some(count),
            SizeLimit::Off => None,
        }
    }
}

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

/// A whole number of 1,000 or more: ASCII digits and nothing else, after a
/// trim. A number too large for `usize` gives `usize::MAX`. The error never
/// holds the text.
pub fn parse_size_limit(text: &str, source: SettingSource) -> Result<usize, ConfigError> {
    let text = text.trim();
    let bad = ConfigError::SizeLimitBad { source };
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(bad);
    }
    let count = text.parse::<usize>().unwrap_or(usize::MAX);
    if count < MIN_SIZE_LIMIT {
        return Err(bad);
    }
    Ok(count)
}

/// The option, else `JAG_MAX_CHARS`, else the default. A blank variable counts
/// as not set.
pub fn size_limit(env: &Env, option: Option<SizeLimit>) -> Result<SizeLimit, ConfigError> {
    if let Some(option) = option {
        return Ok(option);
    }
    match (env.vars)(SIZE_LIMIT_VARIABLE) {
        None => Ok(SizeLimit::Chars(DEFAULT_SIZE_LIMIT)),
        Some(value) => {
            let bad = ConfigError::SizeLimitBad {
                source: SettingSource::Variable,
            };
            let text = value.to_str().ok_or(bad)?;
            if text.trim().is_empty() {
                return Ok(SizeLimit::Chars(DEFAULT_SIZE_LIMIT));
            }
            parse_size_limit(text, SettingSource::Variable).map(SizeLimit::Chars)
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

    fn size(pairs: &[(&str, &str)], option: Option<SizeLimit>) -> Result<SizeLimit, ConfigError> {
        let vars = vars(pairs);
        size_limit(
            &Env {
                vars: &vars,
                home: Some(PathBuf::from("/nowhere")),
            },
            option,
        )
    }

    fn size_bad(source: SettingSource) -> ConfigError {
        ConfigError::SizeLimitBad { source }
    }

    #[test]
    fn size_limits_of_1000_or_more_pass_for_both_sources() {
        for (text, count) in [
            ("1000", 1_000),
            ("20000", 20_000),
            ("01000", 1_000),
            (" 40000 ", 40_000),
            ("99999999999999999999999", usize::MAX),
        ] {
            for source in SOURCES {
                assert_eq!(parse_size_limit(text, source), Ok(count), "{text:?}");
            }
        }
    }

    #[test]
    fn other_size_limits_are_refused_for_both_sources() {
        for text in [
            "0",
            "999",
            "0999",
            "-1",
            "+1000",
            "1e4",
            "20,000",
            "1000.0",
            "20000 chars",
            "off",
            "none",
            "abc",
            "",
            "  ",
        ] {
            for source in SOURCES {
                assert_eq!(
                    parse_size_limit(text, source),
                    Err(size_bad(source)),
                    "{text:?}"
                );
            }
        }
    }

    #[test]
    fn the_size_limit_defaults_to_20000_and_the_smallest_is_1000() {
        assert_eq!(size(&[], None), Ok(SizeLimit::Chars(20_000)));
        assert_eq!(DEFAULT_SIZE_LIMIT, 20_000);
        assert_eq!(MIN_SIZE_LIMIT, 1_000);
        assert_eq!(SizeLimit::Chars(5).chars(), Some(5));
        assert_eq!(SizeLimit::Off.chars(), None);
    }

    #[test]
    fn the_size_option_wins_over_the_variable_and_hides_a_bad_one() {
        let variable = (SIZE_LIMIT_VARIABLE, "40000");
        assert_eq!(
            size(&[variable], Some(SizeLimit::Chars(2_000))),
            Ok(SizeLimit::Chars(2_000))
        );
        assert_eq!(
            size(&[(SIZE_LIMIT_VARIABLE, "abc")], Some(SizeLimit::Off)),
            Ok(SizeLimit::Off)
        );
    }

    #[test]
    fn the_size_variable_is_used_and_a_blank_one_is_not_set() {
        assert_eq!(
            size(&[(SIZE_LIMIT_VARIABLE, " 40000 ")], None),
            Ok(SizeLimit::Chars(40_000))
        );
        assert_eq!(
            size(&[(SIZE_LIMIT_VARIABLE, " ")], None),
            Ok(SizeLimit::Chars(DEFAULT_SIZE_LIMIT))
        );
        assert_eq!(
            size(&[(SIZE_LIMIT_VARIABLE, "999")], None),
            Err(size_bad(SettingSource::Variable))
        );
    }

    #[test]
    fn a_size_variable_that_is_not_text_is_refused() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let vars = |name: &str| {
            (name == SIZE_LIMIT_VARIABLE).then(|| OsString::from_vec(vec![0xff, 0xfe]))
        };
        let env = Env {
            vars: &vars,
            home: Some(PathBuf::from("/nowhere")),
        };
        assert_eq!(
            size_limit(&env, None),
            Err(size_bad(SettingSource::Variable))
        );
    }
}
