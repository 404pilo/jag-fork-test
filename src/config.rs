//! C-config: the API key, its lookup, its store, and its removal.
//!
//! Only this module reads the key. The `ApiKey` type never prints itself,
//! and jag never writes the text of a YAML error because serde errors can
//! quote a value (spec decision 19).

mod change;
mod fs;
mod setup;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::failure::Failure;

pub use change::{ChangeState, load_for_change, write_key_file};
pub use fs::{Fs, OpenFile, RealFs};
pub use setup::{Prompter, TtyPrompter, forget_key, setup};

/// The page where a user gets a key (DECISION-09).
pub const KEYS_PAGE: &str = "https://console.typesafe.ai/keys";
pub const JAG_VARIABLE: &str = "JAG_TYPESAFE_API_KEY";
pub const TYPESAFE_VARIABLE: &str = "TYPESAFE_API_KEY";

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
}

impl KeySource {
    fn variable(&self) -> Option<&'static str> {
        match self {
            KeySource::JagVariable => Some(JAG_VARIABLE),
            KeySource::TypeSafeVariable => Some(TYPESAFE_VARIABLE),
            KeySource::ConfigFile(_) => None,
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

    /// The first key variable that is set and not blank.
    pub(crate) fn set_variable(&self) -> Option<&'static str> {
        [JAG_VARIABLE, TYPESAFE_VARIABLE]
            .into_iter()
            .find(|name| (self.vars)(name).is_some_and(|v| !is_blank(&v)))
    }
}

/// Each cause of a config failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    KeyMissing,
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
    KeyTypedBad,
    PromptFailed {
        kind: std::io::ErrorKind,
    },
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
            ConfigError::KeyFormatBad { source } => match source.variable() {
                Some(name) => (
                    format!(
                        "the key in the variable {name} has a space or a character that is not plain ASCII."
                    ),
                    format!("Set {name} to the key from {KEYS_PAGE}, or unset it."),
                    3,
                ),
                None => {
                    let KeySource::ConfigFile(path) = source else {
                        unreachable!("a variable source has a name")
                    };
                    (
                        format!(
                            "the key in {} has a space or a character that is not plain ASCII.",
                            path.display()
                        ),
                        "Run `jag --setup` to store the key again.".to_owned(),
                        3,
                    )
                }
            },
            ConfigError::KeyFileNotRegular { path } => (
                format!("{} is not a regular file.", path.display()),
                format!("Remove it, then run `jag --setup`, or set {JAG_VARIABLE}."),
                3,
            ),
            ConfigError::KeyFileWrite { path, kind } => (
                format!("jag could not change {} ({kind}).", path.display()),
                "Check that you can write to the folder.".to_owned(),
                3,
            ),
            ConfigError::NoHome => (
                "HOME is not set, so jag has no folder for the key file.".to_owned(),
                format!("Set HOME, or set {JAG_VARIABLE}."),
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
                    "{} holds settings other than config.api-key.",
                    path.display()
                ),
                "Remove the other settings, or edit the file by hand.".to_owned(),
                2,
            ),
            ConfigError::NoTerminal => (
                "--setup needs a terminal to read the key.".to_owned(),
                format!("To give the key with no terminal, set {JAG_VARIABLE}."),
                2,
            ),
            ConfigError::KeyTypedBad => (
                "the key has a space or a character that is not plain ASCII.".to_owned(),
                format!("Copy the key again from {KEYS_PAGE}."),
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

pub(crate) fn read_open_file(file: &mut dyn OpenFile, path: &Path) -> Result<String, ConfigError> {
    file.read_to_string()
        .map_err(|_| ConfigError::KeyFileUnreadable {
            path: path.to_owned(),
        })
}

/// Find the key (decision 7): `JAG_TYPESAFE_API_KEY`, then
/// `TYPESAFE_API_KEY`, then `~/.config/jag/config.yml`. The first set source
/// wins. A source that is set but bad stops the lookup (decision 15).
pub fn find_key(env: &Env, fs: &dyn Fs) -> Result<(ApiKey, KeySource), ConfigError> {
    for (name, source) in [
        (JAG_VARIABLE, KeySource::JagVariable),
        (TYPESAFE_VARIABLE, KeySource::TypeSafeVariable),
    ] {
        if let Some(raw) = (env.vars)(name).filter(|value| !is_blank(value)) {
            // A value that is not UTF-8 has a bad format.
            return match raw.to_str().ok_or(KeyFormatError).and_then(ApiKey::parse) {
                Ok(key) => Ok((key, source)),
                Err(KeyFormatError) => Err(ConfigError::KeyFormatBad { source }),
            };
        }
    }
    if let Some(path) = env.key_path()
        && let Some(mut file) = open_key_file(fs, &path)?
    {
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
        if let Some(raw) = file_key_text(&parsed) {
            let source = KeySource::ConfigFile(path);
            return match ApiKey::parse(raw) {
                Ok(key) => Ok((key, source)),
                Err(KeyFormatError) => Err(ConfigError::KeyFormatBad { source }),
            };
        }
    }
    Err(ConfigError::KeyMissing)
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
mod tests {
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
        find_key(&env, &RealFs)
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
            let error = find_key(&env, &RealFs).err().unwrap();
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
            let error = find_key(
                &Env {
                    vars: &|_: &str| None,
                    home: Some(home.path().to_owned()),
                },
                &RealFs,
            )
            .err()
            .unwrap();
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
