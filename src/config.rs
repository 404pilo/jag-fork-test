//! C-config: the API key, its lookup, its store, and its removal.
//!
//! Only this module reads the key. The `ApiKey` type never prints itself,
//! and jag never writes the text of a YAML error because serde errors can
//! quote a value (spec decision 19).

mod change;
mod fs;
mod limit;
mod model;
mod setup;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::failure::Failure;

pub use change::{ChangeState, FileValues, load_for_change, write_key_file};
pub use fs::{Fs, OpenFile, RealFs};
pub use limit::{
    COST_LIMIT_OPTION, COST_LIMIT_VARIABLE, DEFAULT_COST_LIMIT, DEFAULT_SIZE_LIMIT, MAX_COST_LIMIT,
    MIN_SIZE_LIMIT, SIZE_LIMIT_OFF_OPTION, SIZE_LIMIT_OPTION, SIZE_LIMIT_VARIABLE, SizeLimit,
    cost_limit, parse_cost_limit, parse_size_limit, size_limit,
};
pub use model::{
    BASE_URL_OPTION, BASE_URL_VARIABLE, DEFAULT_BASE_URL, DEFAULT_MODEL, MODEL_OPTION,
    MODEL_VARIABLE, ModelSettings, Origin, Route, SettingOptions, SettingSource, TimeSetting,
    Times, model_settings, normalize_base_url, parse_seconds, price_of, route_of,
};
pub use setup::{Prompter, TtyPrompter, forget_key, setup};

/// The page where a user gets a key (DECISION-09).
pub const KEYS_PAGE: &str = "https://console.typesafe.ai/keys";
pub const JAG_VARIABLE: &str = "JAG_TYPESAFE_API_KEY";
pub const TYPESAFE_VARIABLE: &str = "TYPESAFE_API_KEY";
pub const GATEWAY_VARIABLE: &str = "JAG_GATEWAY_API_KEY";

/// A TypeSafe API key. The text stays private.
///
/// No `Clone`, no `Serialize`, and no `Deref` to `str`.
pub struct ApiKey(String);

/// The key has a bad format. The error never holds the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyFormatError;

impl ApiKey {
    /// Trim the text, then check decision 16: one or more printable ASCII
    /// characters (`0x21` to `0x7E`) and no space.
    pub fn parse(raw: &str) -> Result<ApiKey, KeyFormatError> {
        let text = raw.trim();
        if !text.is_empty() && text.bytes().all(|b| (0x21..=0x7E).contains(&b)) {
            Ok(ApiKey(text.to_owned()))
        } else {
            Err(KeyFormatError)
        }
    }

    /// The text of the key. Only the config module and the model client
    /// (FEATURE-04) call this.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(***)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Where `find_key` found the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    JagVariable,
    TypeSafeVariable,
    ConfigFile(PathBuf),
    /// `JAG_GATEWAY_API_KEY`.
    GatewayVariable,
    /// `config.gateway-api-key` in the file.
    GatewayConfigFile(PathBuf),
}

impl KeySource {
    fn variable(&self) -> Option<&'static str> {
        match self {
            KeySource::JagVariable => Some(JAG_VARIABLE),
            KeySource::TypeSafeVariable => Some(TYPESAFE_VARIABLE),
            KeySource::GatewayVariable => Some(GATEWAY_VARIABLE),
            KeySource::ConfigFile(_) | KeySource::GatewayConfigFile(_) => None,
        }
    }
}

/// The environment, as an argument, so that tests give fakes (decision 17).
pub struct Env<'a> {
    /// The value of a variable. The value can be bad UTF-8: that is an error,
    /// not "not set" (decision 15).
    pub vars: &'a dyn Fn(&str) -> Option<OsString>,
    pub home: Option<PathBuf>,
}

impl Env<'_> {
    /// `<home>/.config/jag/config.yml`. `None` when there is no `$HOME`.
    pub fn key_path(&self) -> Option<PathBuf> {
        let home = self.home.as_deref().filter(|h| !h.as_os_str().is_empty())?;
        Some(home.join(".config").join("jag").join("config.yml"))
    }

    /// The first key variable of the route that is set and not blank.
    pub(crate) fn set_key_variable(&self, route: Route) -> Option<&'static str> {
        let names: &[&'static str] = match route {
            Route::Direct => &[JAG_VARIABLE, TYPESAFE_VARIABLE],
            Route::Gateway => &[GATEWAY_VARIABLE],
        };
        names
            .iter()
            .copied()
            .find(|name| (self.vars)(name).is_some_and(|v| !is_blank(&v)))
    }
}

/// Each cause of a config failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    KeyMissing,
    /// No key for the gateway route (decision 11). `by_option` is true when
    /// the base URL came from `--base-url`.
    GatewayKeyMissing {
        by_option: bool,
    },
    KeyFileOpen {
        path: PathBuf,
        mode: u32,
    },
    KeyFileUnreadable {
        path: PathBuf,
    },
    KeyFileBad {
        path: PathBuf,
        line: Option<u64>,
    },
    KeyFormatBad {
        source: KeySource,
    },
    KeyFileNotRegular {
        path: PathBuf,
    },
    KeyFileWrite {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    NoHome,
    KeyFileIsLink {
        path: PathBuf,
    },
    KeyFileHasOtherKeys {
        path: PathBuf,
    },
    NoTerminal,
    KeyTypedBad {
        route: Route,
    },
    PromptFailed {
        kind: std::io::ErrorKind,
    },
    /// A base URL of an option or a variable is not an allowed address. The
    /// text never shows it.
    BaseUrlBad {
        source: SettingSource,
    },
    /// `config.base-url` is not an allowed address. The text never shows it.
    FileBaseUrlBad {
        path: PathBuf,
    },
    /// A model name of an option is blank, or `JAG_MODEL` is not text. The
    /// text never shows it.
    ModelBad {
        source: SettingSource,
    },
    /// A time option or a time variable is not a whole number of seconds
    /// from 1 to 60. The text never shows the value.
    TimeBad {
        setting: TimeSetting,
        source: SettingSource,
    },
    /// The cost limit is not a whole number from 1 to 2,000. The text never
    /// shows the value.
    CostLimitBad {
        source: SettingSource,
    },
    /// The size limit is not a whole number of 1,000 or more. The text never
    /// shows the value.
    SizeLimitBad {
        source: SettingSource,
    },
}

impl ConfigError {
    /// True for KeyFileOpen, KeyFileUnreadable, KeyFileNotRegular, and
    /// KeyFileBad: the errors of a read of the config file.
    pub fn is_file_error(&self) -> bool {
        matches!(
            self,
            ConfigError::KeyFileOpen { .. }
                | ConfigError::KeyFileUnreadable { .. }
                | ConfigError::KeyFileNotRegular { .. }
                | ConfigError::KeyFileBad { .. }
        )
    }
}

impl From<ConfigError> for Failure {
    fn from(error: ConfigError) -> Failure {
        let (cause, fix, exit_code) = match error {
            ConfigError::KeyMissing => (
                "no TypeSafe API key found.".to_owned(),
                format!(
                    "Get a key at {KEYS_PAGE}, then run `jag --setup`,\nor set {JAG_VARIABLE}."
                ),
                3,
            ),
            ConfigError::GatewayKeyMissing { by_option: false } => (
                "no gateway key found.".to_owned(),
                format!(
                    "Get a key from the gateway, then run `jag --setup`,\nor set {GATEWAY_VARIABLE}."
                ),
                3,
            ),
            ConfigError::GatewayKeyMissing { by_option: true } => (
                "no gateway key found.".to_owned(),
                format!("Set {GATEWAY_VARIABLE} to the key of the gateway."),
                3,
            ),
            ConfigError::KeyFileOpen { path, mode } => (
                format!(
                    "{} can be read by other users (mode {:04o}).",
                    path.display(),
                    mode & 0o777
                ),
                format!("Run `chmod 600 {}`.", path.display()),
                3,
            ),
            ConfigError::KeyFileUnreadable { path } => (
                format!("jag cannot read {}.", path.display()),
                format!("Check the permissions of {}.", path.display()),
                3,
            ),
            ConfigError::KeyFileBad { path, line } => (
                match line {
                    Some(n) => format!("{} is not valid YAML (line {n}).", path.display()),
                    None => format!("{} is not valid YAML.", path.display()),
                },
                "Fix the file by hand, or remove it and run `jag --setup`.".to_owned(),
                3,
            ),
            ConfigError::KeyFormatBad { source } => match source {
                KeySource::GatewayVariable => (
                    format!(
                        "the key in the variable {GATEWAY_VARIABLE} has a space or a character that is not plain ASCII."
                    ),
                    format!("Set {GATEWAY_VARIABLE} to the key of the gateway, or unset it."),
                    3,
                ),
                KeySource::GatewayConfigFile(path) => (
                    format!(
                        "the gateway key in {} has a space or a character that is not plain ASCII.",
                        path.display()
                    ),
                    format!(
                        "Fix config.gateway-api-key in {}, or set {GATEWAY_VARIABLE}.",
                        path.display()
                    ),
                    3,
                ),
                KeySource::ConfigFile(path) => (
                    format!(
                        "the key in {} has a space or a character that is not plain ASCII.",
                        path.display()
                    ),
                    "Run `jag --setup` to store the key again.".to_owned(),
                    3,
                ),
                KeySource::JagVariable | KeySource::TypeSafeVariable => {
                    let name = source.variable().unwrap_or(JAG_VARIABLE);
                    (
                        format!(
                            "the key in the variable {name} has a space or a character that is not plain ASCII."
                        ),
                        format!("Set {name} to the key from {KEYS_PAGE}, or unset it."),
                        3,
                    )
                }
            },
            ConfigError::KeyFileNotRegular { path } => (
                format!("{} is not a regular file.", path.display()),
                "Remove it, then run `jag --setup`.".to_owned(),
                3,
            ),
            ConfigError::KeyFileWrite { path, kind } => (
                format!("jag could not change {} ({kind}).", path.display()),
                "Check that you can write to the folder.".to_owned(),
                3,
            ),
            ConfigError::NoHome => (
                "HOME is not set, so jag has no folder for the key file.".to_owned(),
                format!(
                    "Set HOME, or set {JAG_VARIABLE},\nor {GATEWAY_VARIABLE} for an AI gateway."
                ),
                2,
            ),
            ConfigError::KeyFileIsLink { path } => (
                format!("{} is a symbolic link.", path.display()),
                format!(
                    "Edit the target of {} by hand, or replace the link with a file.",
                    path.display()
                ),
                2,
            ),
            ConfigError::KeyFileHasOtherKeys { path } => (
                format!(
                    "{} holds settings other than config.api-key, config.gateway-api-key, config.base-url, and config.model.",
                    path.display()
                ),
                "Remove the other settings, or edit the file by hand.".to_owned(),
                2,
            ),
            ConfigError::NoTerminal => (
                "--setup needs a terminal to read the key.".to_owned(),
                format!(
                    "To give the key with no terminal, set {JAG_VARIABLE},\nor {GATEWAY_VARIABLE} for an AI gateway."
                ),
                2,
            ),
            ConfigError::KeyTypedBad { route } => (
                "the key has a space or a character that is not plain ASCII.".to_owned(),
                match route {
                    Route::Direct => format!("Copy the key again from {KEYS_PAGE}."),
                    Route::Gateway => "Copy the key again from the gateway.".to_owned(),
                },
                2,
            ),
            ConfigError::PromptFailed {
                kind: std::io::ErrorKind::Interrupted,
            } => (
                "setup was interrupted, and it stored nothing.".to_owned(),
                "Run `jag --setup` again when you are ready.".to_owned(),
                2,
            ),
            ConfigError::PromptFailed { kind } => (
                format!("jag could not read your answer ({kind})."),
                "Run `jag --setup` again in a terminal.".to_owned(),
                2,
            ),
            ConfigError::BaseUrlBad {
                source: SettingSource::Option,
            } => (
                format!("the value of {BASE_URL_OPTION} is not an https address."),
                format!("Use an https address, such as {BASE_URL_OPTION} https://api.typesafe.ai."),
                2,
            ),
            ConfigError::BaseUrlBad {
                source: SettingSource::Variable,
            } => (
                format!("{BASE_URL_VARIABLE} is not an https address."),
                format!("Set {BASE_URL_VARIABLE} to an https address, or unset it."),
                2,
            ),
            ConfigError::FileBaseUrlBad { path } => (
                format!(
                    "config.base-url in {} is not an https address.",
                    path.display()
                ),
                format!(
                    "Set config.base-url in {} to an https address, or remove it.",
                    path.display()
                ),
                2,
            ),
            ConfigError::ModelBad {
                source: SettingSource::Option,
            } => (
                format!("the value of {MODEL_OPTION} is blank."),
                format!("Use a model name, such as {MODEL_OPTION} jev-1.13.0."),
                2,
            ),
            ConfigError::ModelBad {
                source: SettingSource::Variable,
            } => (
                format!("{MODEL_VARIABLE} is not text."),
                format!("Set {MODEL_VARIABLE} to a model name, or unset it."),
                2,
            ),
            ConfigError::CostLimitBad { source } => match source {
                SettingSource::Option => (
                    format!(
                        "the value of {COST_LIMIT_OPTION} is not a whole number from 1 to 2,000."
                    ),
                    format!("Use a number such as {COST_LIMIT_OPTION} 400."),
                    2,
                ),
                SettingSource::Variable => (
                    format!("{COST_LIMIT_VARIABLE} is not a whole number from 1 to 2,000."),
                    format!("Set {COST_LIMIT_VARIABLE} to a number such as 400, or unset it."),
                    2,
                ),
            },
            ConfigError::SizeLimitBad { source } => match source {
                SettingSource::Option => (
                    format!(
                        "the value of {SIZE_LIMIT_OPTION} is not a whole number of 1,000 or more."
                    ),
                    format!(
                        "Use a number such as {SIZE_LIMIT_OPTION} 40000, or turn the limit off with {SIZE_LIMIT_OFF_OPTION}."
                    ),
                    2,
                ),
                SettingSource::Variable => (
                    format!("{SIZE_LIMIT_VARIABLE} is not a whole number of 1,000 or more."),
                    format!("Set {SIZE_LIMIT_VARIABLE} to a number such as 40000, or unset it."),
                    2,
                ),
            },
            ConfigError::TimeBad { setting, source } => {
                let (name, option, example) = match (setting, source) {
                    (TimeSetting::Timeout, SettingSource::Option) => {
                        ("the value of --timeout", "--timeout", 10)
                    }
                    (TimeSetting::Timeout, SettingSource::Variable) => {
                        ("JAG_TIMEOUT", "JAG_TIMEOUT", 10)
                    }
                    (TimeSetting::RetryWait, SettingSource::Option) => {
                        ("the value of --retry-wait", "--retry-wait", 2)
                    }
                    (TimeSetting::RetryWait, SettingSource::Variable) => {
                        ("JAG_RETRY_WAIT", "JAG_RETRY_WAIT", 2)
                    }
                };
                let fix = match source {
                    SettingSource::Option => format!("Use a number such as {option} {example}."),
                    SettingSource::Variable => {
                        format!("Set {option} to a number such as {example}, or unset it.")
                    }
                };
                (
                    format!("{name} is not a whole number of seconds from 1 to 60."),
                    fix,
                    2,
                )
            }
        };
        Failure {
            cause,
            fix,
            exit_code,
        }
    }
}

// The file form (decision 8). A search reads the loose structs and ignores
// other keys (decision 18). A change reads the strict copies too.
#[derive(Deserialize, Default)]
pub(crate) struct File {
    pub(crate) config: Option<Section>,
}

#[derive(Deserialize, Default)]
pub(crate) struct Section {
    #[serde(rename = "api-key")]
    pub(crate) api_key: Option<String>,
    #[serde(rename = "gateway-api-key")]
    pub(crate) gateway_api_key: Option<String>,
    #[serde(rename = "base-url")]
    pub(crate) base_url: Option<String>,
    pub(crate) model: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrictFile {
    #[allow(dead_code)]
    config: Option<StrictSection>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrictSection {
    #[serde(rename = "api-key")]
    #[allow(dead_code)]
    api_key: Option<String>,
    #[serde(rename = "gateway-api-key")]
    #[allow(dead_code)]
    gateway_api_key: Option<String>,
    #[serde(rename = "base-url")]
    #[allow(dead_code)]
    base_url: Option<String>,
    #[allow(dead_code)]
    model: Option<String>,
}

/// True when the text holds more than blank lines, comments, and a document marker.
pub(crate) fn has_yaml_content(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        !line.is_empty() && !line.starts_with('#') && line != "---"
    })
}

/// Parse with the loose structs. A YAML error gives the path and the line
/// only, never the error text (decision 19).
pub(crate) fn parse_loose(text: &str, path: &Path) -> Result<File, ConfigError> {
    if !has_yaml_content(text) {
        return Ok(File::default());
    }
    serde_saphyr::from_str::<File>(text).map_err(|error| ConfigError::KeyFileBad {
        path: path.to_owned(),
        line: error.location().map(|location| location.line()),
    })
}

/// The key text of the file, or `None` when it is missing or blank.
pub(crate) fn file_key_text(file: &File) -> Option<&str> {
    file.config
        .as_ref()?
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

/// The gateway key text of the file, or `None` when it is missing or blank.
pub(crate) fn file_gateway_key_text(file: &File) -> Option<&str> {
    file.config
        .as_ref()?
        .gateway_api_key
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

/// A variable is blank when it is UTF-8 and holds only white space.
fn is_blank(value: &OsStr) -> bool {
    value.to_str().is_some_and(|text| text.trim().is_empty())
}

/// Open the key file, or `None` when it does not exist. A file that is not a
/// regular file (a pipe, for example) is refused before the open, so that
/// the open does not wait.
pub(crate) fn open_key_file(
    fs: &dyn Fs,
    path: &Path,
) -> Result<Option<Box<dyn OpenFile>>, ConfigError> {
    let unreadable = || ConfigError::KeyFileUnreadable {
        path: path.to_owned(),
    };
    if fs.is_special(path).map_err(|_| unreadable())? {
        return Err(ConfigError::KeyFileNotRegular {
            path: path.to_owned(),
        });
    }
    match fs.open(path) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(unreadable()),
    }
}

/// The path and the mode (`mode & 0o777`) of the key file, or `None` when
/// there is no `$HOME` or no file. jag never reads the text of the file here
/// (FEATURE-02, decision 9).
pub(crate) fn key_file_mode(env: &Env, fs: &dyn Fs) -> Result<Option<(PathBuf, u32)>, ConfigError> {
    let Some(path) = env.key_path() else {
        return Ok(None);
    };
    let Some(file) = open_key_file(fs, &path)? else {
        return Ok(None);
    };
    let mode = file
        .mode()
        .map_err(|_| ConfigError::KeyFileUnreadable { path: path.clone() })?;
    Ok(Some((path, mode & 0o777)))
}

pub(crate) fn read_open_file(file: &mut dyn OpenFile, path: &Path) -> Result<String, ConfigError> {
    file.read_to_string()
        .map_err(|_| ConfigError::KeyFileUnreadable {
            path: path.to_owned(),
        })
}

/// The config file of one run. The settings step and the key step share the
/// read (decision 12).
pub struct ConfigFile<'a> {
    env: &'a Env<'a>,
    fs: &'a dyn Fs,
    read: std::cell::OnceCell<Result<Option<(PathBuf, File)>, ConfigError>>,
}

impl<'a> ConfigFile<'a> {
    pub fn new(env: &'a Env<'a>, fs: &'a dyn Fs) -> Self {
        Self {
            env,
            fs,
            read: std::cell::OnceCell::new(),
        }
    }

    /// The path and the parse of the file. `Ok(None)` when there is no home
    /// or no file. The first call opens the file, checks the mode of the open
    /// file, reads it, and parses it. Each later call gives the same result
    /// and reads nothing.
    pub(crate) fn get(&self) -> Result<Option<(&Path, &File)>, ConfigError> {
        match self.read.get_or_init(|| self.load()) {
            Ok(Some((path, file))) => Ok(Some((path.as_path(), file))),
            Ok(None) => Ok(None),
            Err(error) => Err(error.clone()),
        }
    }

    pub(crate) fn fs(&self) -> &dyn Fs {
        self.fs
    }

    fn load(&self) -> Result<Option<(PathBuf, File)>, ConfigError> {
        let Some(path) = self.env.key_path() else {
            return Ok(None);
        };
        let Some(mut file) = open_key_file(self.fs, &path)? else {
            return Ok(None);
        };
        // Check the mode of the open file before jag reads a byte.
        let mode = file
            .mode()
            .map_err(|_| ConfigError::KeyFileUnreadable { path: path.clone() })?;
        if mode & 0o077 != 0 {
            return Err(ConfigError::KeyFileOpen {
                path,
                mode: mode & 0o777,
            });
        }
        let text = read_open_file(file.as_mut(), &path)?;
        let parsed = parse_loose(&text, &path)?;
        Ok(Some((path, parsed)))
    }
}

/// The key from one variable, when it is set and not blank. A value with a
/// bad format gives `KeyFormatBad`.
fn key_from_variable(
    env: &Env,
    name: &str,
    source: KeySource,
) -> Result<Option<(ApiKey, KeySource)>, ConfigError> {
    let Some(raw) = (env.vars)(name).filter(|value| !is_blank(value)) else {
        return Ok(None);
    };
    // A value that is not UTF-8 has a bad format.
    match raw.to_str().ok_or(KeyFormatError).and_then(ApiKey::parse) {
        Ok(key) => Ok(Some((key, source))),
        Err(KeyFormatError) => Err(ConfigError::KeyFormatBad { source }),
    }
}

/// The key sources of a route: the variables, the reader of the file, the
/// source of a key of the file, and the error for no key.
type RouteSources<'a> = (
    &'a [(&'a str, KeySource)],
    fn(&File) -> Option<&str>,
    fn(PathBuf) -> KeySource,
    ConfigError,
);

/// Find the key of the route (decision 11). The first set source wins. A
/// source that is set but bad stops the lookup. jag never reads a source of
/// the other route.
///
/// - Direct: `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, then `config.api-key`.
/// - Gateway: `JAG_GATEWAY_API_KEY`, then `config.gateway-api-key`.
pub fn find_key(
    env: &Env,
    file: &ConfigFile,
    route: Route,
    base_url_origin: &Origin,
) -> Result<(ApiKey, KeySource), ConfigError> {
    let (variables, file_text, file_source, missing): RouteSources = match route {
        Route::Direct => (
            &[
                (JAG_VARIABLE, KeySource::JagVariable),
                (TYPESAFE_VARIABLE, KeySource::TypeSafeVariable),
            ],
            file_key_text,
            KeySource::ConfigFile,
            ConfigError::KeyMissing,
        ),
        Route::Gateway => (
            &[(GATEWAY_VARIABLE, KeySource::GatewayVariable)],
            file_gateway_key_text,
            KeySource::GatewayConfigFile,
            ConfigError::GatewayKeyMissing {
                by_option: matches!(base_url_origin, Origin::Option),
            },
        ),
    };
    for (name, source) in variables {
        if let Some(found) = key_from_variable(env, name, source.clone())? {
            return Ok(found);
        }
    }
    if let Some((path, parsed)) = file.get()?
        && let Some(raw) = file_text(parsed)
    {
        let source = file_source(path.to_owned());
        return match ApiKey::parse(raw) {
            Ok(key) => Ok((key, source)),
            Err(KeyFormatError) => Err(ConfigError::KeyFormatBad { source }),
        };
    }
    Err(missing)
}

#[cfg(test)]
pub(crate) mod testing {
    //! Helpers that the unit tests of this module share.
    use super::*;
    use std::collections::HashMap;

    pub const SENTINEL_KEY: &str = "SENTINEL-SENTINEL-SENTINEL";

    /// True when the tests run as root: a mode 0o000 file is then readable.
    pub fn is_root() -> bool {
        std::fs::read_dir("/root").is_ok()
    }

    /// A fake environment.
    pub fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let map: HashMap<String, OsString> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), OsString::from(*v)))
            .collect();
        move |name| map.get(name).cloned()
    }

    /// Write a key file with the given text and mode under `home`.
    pub fn write_file(home: &Path, text: &str, mode: u32) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let dir = home.join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.yml");
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
        path
    }
}

#[cfg(test)]
mod mode_tests {
    use std::io;

    use super::testing::*;
    use super::*;

    /// An open file that panics when jag reads its text.
    struct NoReadFile(u32);

    impl OpenFile for NoReadFile {
        fn mode(&self) -> io::Result<u32> {
            Ok(self.0)
        }
        fn read_to_string(&mut self) -> io::Result<String> {
            panic!("key_file_mode must not read the text of the file");
        }
    }

    struct NoReadFs(u32);

    impl Fs for NoReadFs {
        fn open(&self, _: &Path) -> io::Result<Box<dyn OpenFile>> {
            Ok(Box::new(NoReadFile(self.0)))
        }
        fn is_symlink(&self, _: &Path) -> io::Result<bool> {
            Ok(false)
        }
        fn is_special(&self, _: &Path) -> io::Result<bool> {
            Ok(false)
        }
        fn create_dir_all(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn create_private_dir(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn create_private_file(&self, _: &Path, _: &str) -> io::Result<()> {
            unreachable!()
        }
        fn rename(&self, _: &Path, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn remove_file(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
    }

    fn mode_of(home: Option<&Path>, fs: &dyn Fs) -> Result<Option<(PathBuf, u32)>, ConfigError> {
        let vars = vars(&[]);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        key_file_mode(&env, fs)
    }

    #[test]
    fn key_file_mode_gives_none_for_no_home() {
        assert_eq!(mode_of(None, &RealFs), Ok(None));
    }

    #[test]
    fn key_file_mode_gives_none_for_a_missing_file() {
        let home = tempfile::tempdir().expect("temporary folder");
        assert_eq!(mode_of(Some(home.path()), &RealFs), Ok(None));
    }

    #[test]
    fn key_file_mode_gives_the_mode_of_a_file() {
        let home = tempfile::tempdir().expect("temporary folder");
        let path = write_file(home.path(), "config: {}\n", 0o644);
        assert_eq!(mode_of(Some(home.path()), &RealFs), Ok(Some((path, 0o644))));
        let path = write_file(home.path(), "config: {}\n", 0o600);
        assert_eq!(mode_of(Some(home.path()), &RealFs), Ok(Some((path, 0o600))));
    }

    #[test]
    fn key_file_mode_refuses_a_folder() {
        let home = tempfile::tempdir().expect("temporary folder");
        let path = home.path().join(".config").join("jag").join("config.yml");
        std::fs::create_dir_all(&path).expect("make folders");
        assert_eq!(
            mode_of(Some(home.path()), &RealFs),
            Err(ConfigError::KeyFileNotRegular { path })
        );
    }

    #[test]
    fn key_file_mode_never_reads_the_text() {
        let result = mode_of(Some(Path::new("/home/user")), &NoReadFs(0o100644));
        assert_eq!(
            result,
            Ok(Some((
                PathBuf::from("/home/user/.config/jag/config.yml"),
                0o644
            )))
        );
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::testing::*;
    use super::*;

    fn find(
        pairs: &[(&str, &str)],
        home: Option<&Path>,
    ) -> Result<(ApiKey, KeySource), ConfigError> {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        find_key(
            &env,
            &ConfigFile::new(&env, &RealFs),
            Route::Direct,
            &Origin::Default,
        )
    }

    fn find_direct(env: &Env) -> Result<(ApiKey, KeySource), ConfigError> {
        find_key(
            env,
            &ConfigFile::new(env, &RealFs),
            Route::Direct,
            &Origin::Default,
        )
    }

    fn key_text(result: Result<(ApiKey, KeySource), ConfigError>) -> (String, KeySource) {
        let (key, source) = result.unwrap();
        (key.expose().to_owned(), source)
    }

    #[test]
    fn key_never_prints_itself() {
        let key = ApiKey::parse(SENTINEL_KEY).unwrap();
        assert_eq!(format!("{key:?}"), "ApiKey(***)");
        assert_eq!(format!("{key}"), "***");
        assert!(!format!("{key:#?}").contains("SENTINEL"));
    }

    #[test]
    fn parse_trims_and_checks_the_format() {
        assert_eq!(ApiKey::parse("  abc  ").unwrap().expose(), "abc");
        assert_eq!(ApiKey::parse("").unwrap_err(), KeyFormatError);
        assert_eq!(ApiKey::parse("   ").unwrap_err(), KeyFormatError);
        assert_eq!(ApiKey::parse("a b").unwrap_err(), KeyFormatError);
        assert_eq!(ApiKey::parse("a\u{e9}b").unwrap_err(), KeyFormatError);
        assert_eq!(ApiKey::parse("a\tb").unwrap_err(), KeyFormatError);
        assert!(ApiKey::parse("~!@#$%^&*()_+-={}[]|:;<>,.?/").is_ok());
    }

    #[test]
    fn the_key_texts_of_each_source_and_route_are_pinned() {
        let path = PathBuf::from("/h/.config/jag/config.yml");
        let gateway_var = KeySource::GatewayVariable;
        let gateway_file = KeySource::GatewayConfigFile(path.clone());
        let cases = [
            (
                ConfigError::KeyFormatBad {
                    source: gateway_var,
                },
                "the key in the variable JAG_GATEWAY_API_KEY has a space or a character that is not plain ASCII.",
                "Set JAG_GATEWAY_API_KEY to the key of the gateway, or unset it.",
            ),
            (
                ConfigError::KeyFormatBad {
                    source: gateway_file,
                },
                "the gateway key in /h/.config/jag/config.yml has a space or a character that is not plain ASCII.",
                "Fix config.gateway-api-key in /h/.config/jag/config.yml, or set JAG_GATEWAY_API_KEY.",
            ),
            (
                ConfigError::KeyTypedBad {
                    route: Route::Direct,
                },
                "the key has a space or a character that is not plain ASCII.",
                "Copy the key again from https://console.typesafe.ai/keys.",
            ),
            (
                ConfigError::KeyTypedBad {
                    route: Route::Gateway,
                },
                "the key has a space or a character that is not plain ASCII.",
                "Copy the key again from the gateway.",
            ),
        ];
        for (error, cause, fix) in cases {
            let failure = Failure::from(error);
            assert_eq!(failure.cause, cause);
            assert_eq!(failure.fix, fix);
        }
    }

    fn find_gateway(
        pairs: &[(&str, &str)],
        home: Option<&Path>,
        origin: &Origin,
    ) -> Result<(ApiKey, KeySource), ConfigError> {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        find_key(
            &env,
            &ConfigFile::new(&env, &RealFs),
            Route::Gateway,
            origin,
        )
    }

    #[test]
    fn the_gateway_route_reads_the_gateway_variable_then_the_gateway_key_of_the_file() {
        let home = tempfile::tempdir().expect("temporary folder");
        let path = write_file(
            home.path(),
            "config:\n  api-key: direct-key\n  gateway-api-key: file-gateway\n",
            0o600,
        );
        let pairs = [(GATEWAY_VARIABLE, "variable-gateway")];
        assert_eq!(
            key_text(find_gateway(&pairs, Some(home.path()), &Origin::Variable)),
            ("variable-gateway".to_owned(), KeySource::GatewayVariable)
        );
        assert_eq!(
            key_text(find_gateway(&[], Some(home.path()), &Origin::Variable)),
            (
                "file-gateway".to_owned(),
                KeySource::GatewayConfigFile(path)
            )
        );
    }

    #[test]
    fn the_gateway_route_never_reads_a_key_of_the_direct_route() {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), "config:\n  api-key: direct-key\n", 0o600);
        let pairs = [(JAG_VARIABLE, "a"), (TYPESAFE_VARIABLE, "b")];
        for (origin, by_option) in [(Origin::Option, true), (Origin::Variable, false)] {
            let error = find_gateway(&pairs, Some(home.path()), &origin)
                .err()
                .unwrap();
            assert_eq!(error, ConfigError::GatewayKeyMissing { by_option });
        }
    }

    #[test]
    fn the_direct_route_never_reads_a_key_of_the_gateway() {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), "config:\n  gateway-api-key: g\n", 0o600);
        let pairs = [(GATEWAY_VARIABLE, "g")];
        assert_eq!(
            find(&pairs, Some(home.path())).err().unwrap(),
            ConfigError::KeyMissing
        );
    }

    #[test]
    fn a_bad_gateway_variable_stops_the_lookup_and_names_the_variable() {
        let pairs = [(GATEWAY_VARIABLE, "has a space")];
        let error = find_gateway(&pairs, None, &Origin::Variable).err().unwrap();
        let ConfigError::KeyFormatBad { source } = &error else {
            panic!("want KeyFormatBad, got {error:?}");
        };
        assert_eq!(source.variable(), Some(GATEWAY_VARIABLE));
        assert_eq!(Failure::from(error).exit_code, 3);
    }

    /// A file system that counts the opens.
    struct CountingFs(std::cell::Cell<u32>);

    impl Fs for CountingFs {
        fn open(&self, path: &Path) -> io::Result<Box<dyn OpenFile>> {
            self.0.set(self.0.get() + 1);
            RealFs.open(path)
        }
        fn is_symlink(&self, path: &Path) -> io::Result<bool> {
            RealFs.is_symlink(path)
        }
        fn is_special(&self, path: &Path) -> io::Result<bool> {
            RealFs.is_special(path)
        }
        fn create_dir_all(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn create_private_dir(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn create_private_file(&self, _: &Path, _: &str) -> io::Result<()> {
            unreachable!()
        }
        fn rename(&self, _: &Path, _: &Path) -> io::Result<()> {
            unreachable!()
        }
        fn remove_file(&self, _: &Path) -> io::Result<()> {
            unreachable!()
        }
    }

    #[test]
    fn the_file_is_read_once_for_the_settings_and_the_key() {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(
            home.path(),
            "config:\n  gateway-api-key: g\n  base-url: https://openrouter.ai/api\n  model: m\n",
            0o600,
        );
        let vars = vars(&[]);
        let env = Env {
            vars: &vars,
            home: Some(home.path().to_owned()),
        };
        let fs = CountingFs(std::cell::Cell::new(0));
        let file = ConfigFile::new(&env, &fs);
        let settings = model_settings(&env, &file, &SettingOptions::default()).unwrap();
        assert_eq!(settings.route, Route::Gateway);
        find_key(&env, &file, settings.route, &settings.base_url_origin).unwrap();
        assert_eq!(fs.0.get(), 1);
    }

    #[test]
    fn a_file_error_is_given_again_and_the_file_is_not_read_again() {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), "config:\n  model: m\n", 0o644);
        let vars = vars(&[]);
        let env = Env {
            vars: &vars,
            home: Some(home.path().to_owned()),
        };
        let fs = CountingFs(std::cell::Cell::new(0));
        let file = ConfigFile::new(&env, &fs);
        for _ in 0..2 {
            assert!(matches!(
                file.get().err().unwrap(),
                ConfigError::KeyFileOpen { mode: 0o644, .. }
            ));
        }
        assert_eq!(fs.0.get(), 1);
    }

    #[test]
    fn no_file_read_happens_when_the_variables_give_everything() {
        let home = tempfile::tempdir().expect("temporary folder");
        write_file(home.path(), "config:\n  model: m\n", 0o644);
        let vars = vars(&[
            (JAG_VARIABLE, "k"),
            ("JAG_BASE_URL", "https://api.typesafe.ai/v1"),
            ("JAG_MODEL", "m"),
        ]);
        let env = Env {
            vars: &vars,
            home: Some(home.path().to_owned()),
        };
        let fs = CountingFs(std::cell::Cell::new(0));
        let file = ConfigFile::new(&env, &fs);
        let settings = model_settings(&env, &file, &SettingOptions::default()).unwrap();
        find_key(&env, &file, settings.route, &settings.base_url_origin).unwrap();
        assert_eq!(fs.0.get(), 0);
    }

    // CRITERION-02: the first set source wins, over the 8 combinations.
    #[test]
    fn the_first_set_source_wins() {
        for bits in 0..8u8 {
            let home = tempfile::tempdir().unwrap();
            let mut pairs = Vec::new();
            if bits & 1 != 0 {
                pairs.push((JAG_VARIABLE, "from-jag"));
            }
            if bits & 2 != 0 {
                pairs.push((TYPESAFE_VARIABLE, "from-typesafe"));
            }
            if bits & 4 != 0 {
                write_file(home.path(), "config:\n  api-key: from-file\n", 0o600);
            }
            let result = find(&pairs, Some(home.path()));
            match bits {
                0 => assert_eq!(result.unwrap_err(), ConfigError::KeyMissing),
                _ if bits & 1 != 0 => {
                    assert_eq!(
                        key_text(result),
                        ("from-jag".into(), KeySource::JagVariable)
                    );
                }
                _ if bits & 2 != 0 => {
                    assert_eq!(
                        key_text(result),
                        ("from-typesafe".into(), KeySource::TypeSafeVariable)
                    );
                }
                _ => {
                    let (text, source) = key_text(result);
                    assert_eq!(text, "from-file");
                    assert!(matches!(source, KeySource::ConfigFile(_)));
                }
            }
        }
    }

    #[test]
    fn a_blank_variable_counts_as_not_set() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: from-file\n", 0o600);
        let pairs = [(JAG_VARIABLE, ""), (TYPESAFE_VARIABLE, "   ")];
        let (text, source) = key_text(find(&pairs, Some(home.path())));
        assert_eq!(text, "from-file");
        assert!(matches!(source, KeySource::ConfigFile(_)));
    }

    // CRITERION-08: a TypeSafe key is enough.
    #[test]
    fn only_the_typesafe_variable_is_enough() {
        let (text, source) = key_text(find(&[(TYPESAFE_VARIABLE, "k1")], None));
        assert_eq!((text.as_str(), source), ("k1", KeySource::TypeSafeVariable));
    }

    #[test]
    fn a_bad_variable_stops_the_lookup() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: from-file\n", 0o600);
        let pairs = [
            (JAG_VARIABLE, "jag test SENTINEL"),
            (TYPESAFE_VARIABLE, "ok"),
        ];
        let error = find(&pairs, Some(home.path())).unwrap_err();
        assert_eq!(
            error,
            ConfigError::KeyFormatBad {
                source: KeySource::JagVariable
            }
        );
        let failure = Failure::from(error);
        assert_eq!(failure.exit_code, 3);
        assert!(failure.cause.contains(JAG_VARIABLE));
        assert!(!failure.render().contains("SENTINEL"));
    }

    #[test]
    fn no_home_and_no_variable_gives_key_missing() {
        assert_eq!(find(&[], None).unwrap_err(), ConfigError::KeyMissing);
        assert_eq!(
            find(&[], Some(Path::new(""))).unwrap_err(),
            ConfigError::KeyMissing
        );
    }

    // CRITERION-06
    #[test]
    fn a_key_file_that_others_can_read_is_refused() {
        for mode in [0o640, 0o604, 0o644, 0o660, 0o777] {
            let home = tempfile::tempdir().unwrap();
            let path = write_file(home.path(), "config:\n  api-key: k\n", mode);
            let error = find(&[], Some(home.path())).unwrap_err();
            assert_eq!(
                error,
                ConfigError::KeyFileOpen {
                    path: path.clone(),
                    mode
                }
            );
            let failure = Failure::from(error);
            assert_eq!(failure.exit_code, 3);
            assert!(
                failure
                    .fix
                    .contains(&format!("chmod 600 {}", path.display()))
            );
        }
        for mode in [0o600, 0o400] {
            let home = tempfile::tempdir().unwrap();
            write_file(home.path(), "config:\n  api-key: k\n", mode);
            assert!(find(&[], Some(home.path())).is_ok());
        }
    }

    #[test]
    fn a_file_with_a_bad_mode_is_refused_before_its_text_is_read() {
        // The text is not YAML. The mode error wins, so jag read no byte.
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "{{{ not yaml", 0o644);
        assert!(matches!(
            find(&[], Some(home.path())).unwrap_err(),
            ConfigError::KeyFileOpen { .. }
        ));
    }

    // CRITERION-07
    #[test]
    fn the_missing_key_text_points_to_the_key_page() {
        let failure = Failure::from(ConfigError::KeyMissing);
        assert_eq!(failure.exit_code, 3);
        let text = failure.render();
        assert!(text.contains("https://console.typesafe.ai/keys"));
        assert!(text.contains("jag --setup"));
        assert!(text.contains(JAG_VARIABLE));
    }

    #[test]
    fn the_file_needs_the_nested_form() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "api-key: top-level\n", 0o600);
        assert_eq!(
            find(&[], Some(home.path())).unwrap_err(),
            ConfigError::KeyMissing
        );
    }

    #[test]
    fn a_search_ignores_other_keys_in_the_file() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: k\nother: 1\n", 0o600);
        assert_eq!(key_text(find(&[], Some(home.path()))).0, "k");
    }

    #[test]
    fn empty_files_and_blank_keys_give_key_missing() {
        for text in [
            "",
            "# only a comment\n",
            "---\n",
            "config:\n  api-key: \"\"\n",
            "config: {}\n",
        ] {
            let home = tempfile::tempdir().unwrap();
            write_file(home.path(), text, 0o600);
            assert_eq!(
                find(&[], Some(home.path())).unwrap_err(),
                ConfigError::KeyMissing,
                "text: {text:?}"
            );
        }
    }

    #[test]
    fn a_bad_format_in_the_file_is_refused_without_the_text() {
        let home = tempfile::tempdir().unwrap();
        write_file(
            home.path(),
            "config:\n  api-key: \"jag test SENTINEL\"\n",
            0o600,
        );
        let error = find(&[], Some(home.path())).unwrap_err();
        assert!(matches!(
            error,
            ConfigError::KeyFormatBad {
                source: KeySource::ConfigFile(_)
            }
        ));
        let failure = Failure::from(error);
        assert_eq!(failure.exit_code, 3);
        assert!(!failure.render().contains("SENTINEL"));
    }

    #[test]
    fn bad_yaml_gives_the_path_and_the_line_never_the_text() {
        let home = tempfile::tempdir().unwrap();
        let path = write_file(home.path(), "config:\n  api-key: [SENTINEL\n", 0o600);
        let error = find(&[], Some(home.path())).unwrap_err();
        let ConfigError::KeyFileBad {
            path: error_path,
            line,
        } = &error
        else {
            panic!("want KeyFileBad, got {error:?}");
        };
        assert_eq!(error_path, &path);
        assert!(line.is_some());
        let failure = Failure::from(error);
        assert_eq!(failure.exit_code, 3);
        assert!(failure.cause.contains("is not valid YAML"));
        assert!(!failure.render().contains("SENTINEL"));
    }

    #[test]
    fn an_unreadable_file_gives_key_file_unreadable() {
        if is_root() {
            return;
        }
        let home = tempfile::tempdir().unwrap();
        let path = write_file(home.path(), "config:\n  api-key: k\n", 0o000);
        assert_eq!(
            find(&[], Some(home.path())).unwrap_err(),
            ConfigError::KeyFileUnreadable { path }
        );
    }

    #[test]
    fn a_link_to_a_private_file_is_followed_and_a_link_to_an_open_file_is_refused() {
        use std::os::unix::fs::symlink;
        let home = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("real.yml");
        std::fs::write(&target, "config:\n  api-key: linked\n").unwrap();
        let dir = home.path().join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        symlink(&target, dir.join("config.yml")).unwrap();
        for (mode, ok) in [(0o600, true), (0o644, false)] {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode)).unwrap();
            let result = find(&[], Some(home.path()));
            if ok {
                assert_eq!(key_text(result).0, "linked");
            } else {
                assert!(matches!(
                    result.unwrap_err(),
                    ConfigError::KeyFileOpen { .. }
                ));
            }
        }
    }

    // Decision 15: a variable that is not UTF-8 is a bad source, not "not set".
    #[test]
    fn a_variable_that_is_not_utf8_stops_the_lookup() {
        use std::os::unix::ffi::OsStringExt;
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: from-file\n", 0o600);
        for name in [JAG_VARIABLE, TYPESAFE_VARIABLE] {
            let bad = |var: &str| (var == name).then(|| OsString::from_vec(vec![b'k', 0xff, 0xfe]));
            let env = Env {
                vars: &bad,
                home: Some(home.path().to_owned()),
            };
            let error = find_direct(&env).err().unwrap();
            let ConfigError::KeyFormatBad { source } = &error else {
                panic!("want KeyFormatBad, got {error:?}");
            };
            assert_eq!(source.variable(), Some(name));
            assert_eq!(Failure::from(error).exit_code, 3);
        }
    }

    // A file that is not a regular file does not make jag wait.
    #[test]
    fn a_directory_or_a_pipe_in_place_of_the_key_file_is_refused() {
        for make_fifo in [false, true] {
            let home = tempfile::tempdir().unwrap();
            let dir = home.path().join(".config").join("jag");
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("config.yml");
            if make_fifo {
                let status = std::process::Command::new("mkfifo")
                    .arg(&path)
                    .status()
                    .unwrap();
                assert!(status.success());
            } else {
                std::fs::create_dir(&path).unwrap();
            }
            let env = Env {
                vars: &|_: &str| None,
                home: Some(home.path().to_owned()),
            };
            let error = find_direct(&env).err().unwrap();
            assert_eq!(error, ConfigError::KeyFileNotRegular { path });
            let failure = Failure::from(error);
            assert_eq!(failure.exit_code, 3);
            assert!(failure.cause.contains("not a regular file"));
        }
    }

    #[test]
    fn an_interrupted_prompt_has_its_own_text() {
        let failure = Failure::from(ConfigError::PromptFailed {
            kind: std::io::ErrorKind::Interrupted,
        });
        assert_eq!(failure.exit_code, 2);
        assert!(failure.cause.contains("interrupted"));
    }
}
