//! Read the key file for a change, and write it.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::{
    ApiKey, ConfigError, Env, Fs, StrictFile, file_key_text, has_yaml_content, open_key_file,
    parse_loose, read_open_file,
};

/// What `--setup` and `--forget-key` find in the key file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeState {
    /// The file exists.
    pub exists: bool,
    /// The mode of the open file. `0` when there is no file.
    pub mode: u32,
    /// The file holds a `config.api-key` that is not blank.
    pub has_key: bool,
}

/// Read the key file for a change. Each error stores nothing and changes
/// nothing. This function does not check the mode (`--setup` does).
pub fn load_for_change(env: &Env, fs: &dyn Fs) -> Result<(PathBuf, ChangeState), ConfigError> {
    // 1. No home.
    let path = env.key_path().ok_or(ConfigError::NoHome)?;
    // 2. A symbolic link (decision 20).
    if fs
        .is_symlink(&path)
        .map_err(|_| ConfigError::KeyFileUnreadable { path: path.clone() })?
    {
        return Err(ConfigError::KeyFileIsLink { path });
    }
    // 3. Open and read from the same handle.
    let Some(mut file) = open_key_file(fs, &path)? else {
        let state = ChangeState {
            exists: false,
            mode: 0,
            has_key: false,
        };
        return Ok((path, state));
    };
    let mode = file
        .mode()
        .map_err(|_| ConfigError::KeyFileUnreadable { path: path.clone() })?;
    let text = read_open_file(file.as_mut(), &path)?;
    // 4. Parse two times (decision 18). The loose parse finds a YAML error.
    //    When it passes, an error of the strict parse means an unknown key.
    let loose = parse_loose(&text, &path)?;
    if has_yaml_content(&text) && serde_saphyr::from_str::<StrictFile>(&text).is_err() {
        return Err(ConfigError::KeyFileHasOtherKeys { path });
    }
    let state = ChangeState {
        exists: true,
        mode,
        has_key: file_key_text(&loose).is_some(),
    };
    Ok((path, state))
}

/// The text of the YAML scalar for the key. A plain key goes as it is. Any
/// other key goes in double quotes, so that it reads back as the same string.
fn yaml_scalar(key: &str) -> String {
    const RESERVED: [&str; 10] = [
        "true", "false", "null", "yes", "no", "on", "off", "y", "n", "nan",
    ];
    let plain = key.starts_with(|c: char| c.is_ascii_alphabetic())
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.-/+=".contains(c))
        && !RESERVED.contains(&key.to_ascii_lowercase().as_str())
        && !key.eq_ignore_ascii_case("inf");
    if plain {
        key.to_owned()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

const TEMP_ATTEMPTS: u32 = 5;

/// A random number from the random seed of the standard library.
fn random_u64() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// Write the key file (decision 8) through a temporary file and a rename.
/// Returns the path of `config.yml`.
pub fn write_key_file(fs: &dyn Fs, home: &Path, key: &ApiKey) -> Result<PathBuf, ConfigError> {
    let write_error = |path: &Path, error: std::io::Error| ConfigError::KeyFileWrite {
        path: path.to_owned(),
        kind: error.kind(),
    };
    let config_dir = home.join(".config");
    let jag_dir = config_dir.join("jag");
    let path = jag_dir.join("config.yml");
    // 1. `~/.config` with the default mode. Never change a mode that exists.
    fs.create_dir_all(&config_dir)
        .map_err(|e| write_error(&config_dir, e))?;
    // 2. `~/.config/jag` with mode 0o700, when it does not exist.
    fs.create_private_dir(&jag_dir)
        .map_err(|e| write_error(&jag_dir, e))?;
    // 3. A new temporary file with mode 0o600. The name holds the process id
    //    and a random part. When the name is taken, try again a few times, so
    //    that a stale temporary file does not block setup.
    let text = format!("config:\n  api-key: {}\n", yaml_scalar(key.expose()));
    let mut attempt = 1;
    let temp = loop {
        let temp = jag_dir.join(format!(
            "config.yml.tmp-{}-{:016x}",
            std::process::id(),
            random_u64()
        ));
        match fs.create_private_file(&temp, &text) {
            Ok(()) => break temp,
            Err(error) if error.kind() == ErrorKind::AlreadyExists && attempt < TEMP_ATTEMPTS => {
                attempt += 1;
            }
            Err(error) => {
                // The file is not ours when the name was taken.
                if error.kind() != ErrorKind::AlreadyExists {
                    let _ = fs.remove_file(&temp);
                }
                return Err(write_error(&path, error));
            }
        }
    };
    // 4. Rename it over `config.yml`.
    if let Err(error) = fs.rename(&temp, &path) {
        let _ = fs.remove_file(&temp);
        return Err(write_error(&path, error));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::config::testing::*;
    use crate::config::{KeySource, RealFs, find_key};

    fn load(home: Option<&Path>) -> Result<(PathBuf, ChangeState), ConfigError> {
        let vars = vars(&[]);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        load_for_change(&env, &RealFs)
    }

    #[test]
    fn no_home_gives_no_home() {
        assert_eq!(load(None).unwrap_err(), ConfigError::NoHome);
        assert_eq!(load(Some(Path::new(""))).unwrap_err(), ConfigError::NoHome);
        assert_eq!(
            crate::failure::Failure::from(ConfigError::NoHome).exit_code,
            2
        );
    }

    #[test]
    fn no_file_and_no_key_and_a_key() {
        let home = tempfile::tempdir().unwrap();
        let (_, state) = load(Some(home.path())).unwrap();
        assert!(!state.exists && !state.has_key);

        write_file(home.path(), "# comment only\n", 0o600);
        let (_, state) = load(Some(home.path())).unwrap();
        assert!(state.exists && !state.has_key);

        write_file(home.path(), "config:\n  api-key: k\n", 0o600);
        let (_, state) = load(Some(home.path())).unwrap();
        assert!(state.exists && state.has_key);
        assert_eq!(state.mode & 0o777, 0o600);
    }

    #[test]
    fn a_bad_yaml_file_gives_key_file_bad() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: [SENTINEL\n", 0o600);
        assert!(matches!(
            load(Some(home.path())).unwrap_err(),
            ConfigError::KeyFileBad { .. }
        ));
    }

    #[test]
    fn other_keys_give_key_file_has_other_keys() {
        for text in [
            "config:\n  api-key: k\nother: 1\n",
            "config:\n  api-key: k\n  other: 1\n",
        ] {
            let home = tempfile::tempdir().unwrap();
            write_file(home.path(), text, 0o600);
            let error = load(Some(home.path())).unwrap_err();
            assert!(
                matches!(error, ConfigError::KeyFileHasOtherKeys { .. }),
                "{text}"
            );
            assert_eq!(crate::failure::Failure::from(error).exit_code, 2);
        }
    }

    #[test]
    fn a_link_gives_key_file_is_link_and_leaves_the_target() {
        use std::os::unix::fs::symlink;
        let home = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("real.yml");
        std::fs::write(&target, "config:\n  api-key: k\n").unwrap();
        let dir = home.path().join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        symlink(&target, dir.join("config.yml")).unwrap();
        assert!(matches!(
            load(Some(home.path())).unwrap_err(),
            ConfigError::KeyFileIsLink { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            "config:\n  api-key: k\n"
        );
    }

    #[test]
    fn an_unreadable_file_gives_key_file_unreadable() {
        if is_root() {
            return;
        }
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: k\n", 0o000);
        assert!(matches!(
            load(Some(home.path())).unwrap_err(),
            ConfigError::KeyFileUnreadable { .. }
        ));
    }

    #[test]
    fn write_creates_the_folders_and_a_private_file() {
        let home = tempfile::tempdir().unwrap();
        let key = ApiKey::parse(SENTINEL_KEY).unwrap();
        let path = write_key_file(&RealFs, home.path(), &key).unwrap();
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        // `~/.config` keeps the default mode, not 0o700.
        assert_ne!(mode(&home.path().join(".config")), 0o700);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            format!("config:\n  api-key: {SENTINEL_KEY}\n")
        );
    }

    #[test]
    fn write_does_not_change_the_mode_of_a_folder_that_exists() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path().join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        write_key_file(&RealFs, home.path(), &ApiKey::parse("k").unwrap()).unwrap();
        assert_eq!(
            std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[test]
    fn every_valid_key_reads_back_as_the_same_text() {
        for text in [
            "plain",
            "sk-abc_123.x/y+z=",
            "true",
            "False",
            "null",
            "~",
            "123",
            "1e3",
            "0x1F",
            "a#b",
            "#start",
            "[x",
            "{x",
            "*x",
            "&x",
            "!x",
            "'q'",
            "\"q\"",
            "a\\b",
            "%x",
            "@x",
            "`x",
            "|x",
            ">x",
            "-x",
            "?x",
            ":x",
            "a:b",
            "a:",
            "yes",
            "No",
            "On",
            "inf",
            ".nan",
        ] {
            let home = tempfile::tempdir().unwrap();
            let key = ApiKey::parse(text).unwrap();
            let path = write_key_file(&RealFs, home.path(), &key).unwrap();
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            let vars = vars(&[]);
            let env = Env {
                vars: &vars,
                home: Some(home.path().to_owned()),
            };
            let (found, source) = find_key(&env, &RealFs).unwrap();
            assert_eq!(found.expose(), text, "key {text:?}");
            assert_eq!(source, KeySource::ConfigFile(path));
        }
    }

    /// A file system with two faults: the rename fails, or the first
    /// `taken` names for a temporary file are already taken.
    struct Faulty {
        rename_fails: bool,
        taken: std::cell::Cell<u32>,
        creates: std::cell::Cell<u32>,
    }

    impl Faulty {
        fn new(rename_fails: bool, taken: u32) -> Faulty {
            Faulty {
                rename_fails,
                taken: std::cell::Cell::new(taken),
                creates: std::cell::Cell::new(0),
            }
        }
    }

    impl Fs for Faulty {
        fn open(&self, path: &Path) -> io::Result<Box<dyn crate::config::OpenFile>> {
            RealFs.open(path)
        }
        fn is_symlink(&self, path: &Path) -> io::Result<bool> {
            RealFs.is_symlink(path)
        }
        fn is_special(&self, path: &Path) -> io::Result<bool> {
            RealFs.is_special(path)
        }
        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            RealFs.create_dir_all(path)
        }
        fn create_private_dir(&self, path: &Path) -> io::Result<()> {
            RealFs.create_private_dir(path)
        }
        fn create_private_file(&self, path: &Path, text: &str) -> io::Result<()> {
            self.creates.set(self.creates.get() + 1);
            if self.taken.get() > 0 {
                self.taken.set(self.taken.get() - 1);
                return Err(io::Error::from(io::ErrorKind::AlreadyExists));
            }
            RealFs.create_private_file(path, text)
        }
        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            if self.rename_fails {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            } else {
                RealFs.rename(from, to)
            }
        }
        fn remove_file(&self, path: &Path) -> io::Result<()> {
            RealFs.remove_file(path)
        }
    }

    #[test]
    fn a_taken_temporary_name_is_tried_again() {
        let home = tempfile::tempdir().unwrap();
        let key = ApiKey::parse("k").unwrap();
        let fs = Faulty::new(false, TEMP_ATTEMPTS - 1);
        write_key_file(&fs, home.path(), &key).unwrap();
        assert_eq!(fs.creates.get(), TEMP_ATTEMPTS);
        // After the retries are spent, setup fails and no file stays.
        let home = tempfile::tempdir().unwrap();
        let fs = Faulty::new(false, TEMP_ATTEMPTS);
        assert!(matches!(
            write_key_file(&fs, home.path(), &key).unwrap_err(),
            ConfigError::KeyFileWrite {
                kind: ErrorKind::AlreadyExists,
                ..
            }
        ));
        assert_eq!(fs.creates.get(), TEMP_ATTEMPTS);
    }

    #[test]
    fn a_stale_temporary_file_does_not_block_setup() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path().join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        let stale = dir.join(format!("config.yml.tmp-{}", std::process::id()));
        std::fs::write(&stale, "stale").unwrap();
        write_key_file(&RealFs, home.path(), &ApiKey::parse("k").unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(&stale).unwrap(), "stale");
        assert!(dir.join("config.yml").exists());
    }

    #[test]
    fn a_failed_rename_gives_key_file_write_and_leaves_no_temporary_file() {
        let home = tempfile::tempdir().unwrap();
        let key = ApiKey::parse(SENTINEL_KEY).unwrap();
        let error = write_key_file(&Faulty::new(true, 0), home.path(), &key).unwrap_err();
        assert!(matches!(
            error,
            ConfigError::KeyFileWrite {
                kind: ErrorKind::PermissionDenied,
                ..
            }
        ));
        let failure = crate::failure::Failure::from(error);
        assert_eq!(failure.exit_code, 3);
        assert!(!failure.render().contains("SENTINEL"));
        let dir = home.path().join(".config").join("jag");
        assert_eq!(std::fs::read_dir(dir).unwrap().count(), 0);
    }

    #[test]
    fn a_failed_write_gives_key_file_write() {
        // `.config` is a file, so the folder cannot be made.
        let home = tempfile::tempdir().unwrap();
        std::fs::write(home.path().join(".config"), "x").unwrap();
        let key = ApiKey::parse("k").unwrap();
        assert!(matches!(
            write_key_file(&RealFs, home.path(), &key).unwrap_err(),
            ConfigError::KeyFileWrite { .. }
        ));
    }
}
