//! `--setup` and `--forget-key`.

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use signal_hook::consts::SIGINT;

use super::{ApiKey, ConfigError, Env, Fs, load_for_change, write_key_file};

/// The questions of setup. A fake gives the answers in a unit test.
pub trait Prompter {
    /// Ask a yes or no question. Only `y` or `yes` (any case) gives `true`.
    fn confirm(&mut self, question: &str) -> io::Result<bool>;
    /// Ask for a secret. The terminal shows no echo.
    fn secret(&mut self, prompt: &str) -> io::Result<String>;
}

/// The production prompter, over `rpassword` and standard input.
pub struct TtyPrompter;

fn is_yes(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

impl Prompter for TtyPrompter {
    fn confirm(&mut self, question: &str) -> io::Result<bool> {
        eprint!("{question}");
        io::stderr().flush()?;
        let mut answer = String::new();
        // End of input gives an empty answer, which keeps the file.
        io::stdin().lock().read_line(&mut answer)?;
        Ok(is_yes(&answer))
    }

    fn secret(&mut self, prompt: &str) -> io::Result<String> {
        // rpassword turns off the echo and the signal keys of the terminal. On
        // Ctrl-C it raises SIGINT, and the default action ends the process
        // before rpassword turns the echo on again. A handler for SIGINT makes
        // `prompt_password` return an `Interrupted` error, so it restores the
        // terminal first.
        //
        // Removing a handler does not bring back the default action. So the
        // first handler stays, and it runs the default action (the end of the
        // process) once `done` is true: a Ctrl-C after the prompt ends jag.
        let done = Arc::new(AtomicBool::new(false));
        let flag = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register_conditional_default(SIGINT, Arc::clone(&done))?;
        let absorb = signal_hook::flag::register(SIGINT, flag)?;
        let result = rpassword::prompt_password(prompt);
        done.store(true, Ordering::SeqCst);
        signal_hook::low_level::unregister(absorb);
        result
    }
}

fn note_variable(env: &Env, out: &mut dyn Write, text: impl Fn(&str) -> String) {
    if let Some(name) = env.set_variable() {
        let _ = writeln!(out, "{}", text(name));
    }
}

/// Store the key. The caller says if standard input is a terminal.
pub fn setup(
    env: &Env,
    fs: &dyn Fs,
    prompter: &mut dyn Prompter,
    out: &mut dyn Write,
    is_terminal: bool,
) -> Result<(), ConfigError> {
    // 1. No terminal: store nothing (CRITERION-72).
    if !is_terminal {
        return Err(ConfigError::NoTerminal);
    }
    // 2. Read the file.
    let (path, state) = load_for_change(env, fs)?;
    // 3. A file that others can read stops setup before the question.
    if state.exists && state.mode & 0o077 != 0 {
        return Err(ConfigError::KeyFileOpen {
            path,
            mode: state.mode & 0o777,
        });
    }
    // 4. Ask before a replace (CRITERION-03).
    if state.has_key {
        let question = format!(
            "A key is already stored in {}. Replace it? [y/N] ",
            path.display()
        );
        let replace = prompter
            .confirm(&question)
            .map_err(|error| ConfigError::PromptFailed { kind: error.kind() })?;
        if !replace {
            let _ = writeln!(out, "Kept the stored key.");
            return Ok(());
        }
    }
    // 5. Read the key with no echo.
    let typed = prompter
        .secret("TypeSafe API key: ")
        .map_err(|error| ConfigError::PromptFailed { kind: error.kind() })?;
    // 6. Check the format. The error holds no text.
    let key = ApiKey::parse(&typed).map_err(|_| ConfigError::KeyTypedBad)?;
    // 7. Write the file. `load_for_change` found a path, so there is a home.
    let home = env.home.as_deref().ok_or(ConfigError::NoHome)?;
    let path = write_key_file(fs, home, &key)?;
    // 8. Report.
    let _ = writeln!(out, "Stored the key in {}.", path.display());
    note_variable(env, out, |name| {
        format!("Note: {name} is set, and it wins over the stored key.")
    });
    Ok(())
}

/// Remove the stored key.
pub fn forget_key(env: &Env, fs: &dyn Fs, out: &mut dyn Write) -> Result<(), ConfigError> {
    // 1. No home: there is no stored key.
    if env.key_path().is_none() {
        let _ = writeln!(out, "No stored key: HOME is not set.");
        return Ok(());
    }
    // 2. Read the file. A bad file stays as it is.
    let (path, state) = load_for_change(env, fs)?;
    // 3. No file, or no key in it.
    if !state.exists || !state.has_key {
        let _ = writeln!(out, "No stored key in {}.", path.display());
        return Ok(());
    }
    // 4. Delete the file. Decision 18 makes sure that it holds only the key.
    fs.remove_file(&path)
        .map_err(|error| ConfigError::KeyFileWrite {
            path: path.clone(),
            kind: error.kind(),
        })?;
    // 5. and 6. Report.
    let _ = writeln!(out, "Removed the stored key from {}.", path.display());
    note_variable(env, out, |name| {
        format!("Note: {name} is still set, so jag still has a key.")
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::config::testing::*;
    use crate::config::{KeySource, RealFs, find_key};
    use crate::failure::Failure;

    /// A prompter with queued answers. It records each call.
    #[derive(Default)]
    struct Fake {
        confirms: VecDeque<bool>,
        secrets: VecDeque<String>,
        calls: usize,
    }

    impl Fake {
        fn secret(text: &str) -> Fake {
            Fake {
                secrets: VecDeque::from([text.to_owned()]),
                ..Fake::default()
            }
        }
        fn answering(confirm: bool, text: &str) -> Fake {
            Fake {
                confirms: VecDeque::from([confirm]),
                ..Fake::secret(text)
            }
        }
    }

    impl Prompter for Fake {
        fn confirm(&mut self, _question: &str) -> io::Result<bool> {
            self.calls += 1;
            Ok(self.confirms.pop_front().expect("an unexpected question"))
        }
        fn secret(&mut self, _prompt: &str) -> io::Result<String> {
            self.calls += 1;
            Ok(self.secrets.pop_front().expect("an unexpected prompt"))
        }
    }

    struct Run {
        result: Result<(), ConfigError>,
        out: String,
        calls: usize,
    }

    fn run_setup(pairs: &[(&str, &str)], home: Option<&Path>, mut fake: Fake, tty: bool) -> Run {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        let mut out = Vec::new();
        let result = setup(&env, &RealFs, &mut fake, &mut out, tty);
        Run {
            result,
            out: String::from_utf8(out).unwrap(),
            calls: fake.calls,
        }
    }

    fn run_forget(
        pairs: &[(&str, &str)],
        home: Option<&Path>,
    ) -> (Result<(), ConfigError>, String) {
        let vars = vars(pairs);
        let env = Env {
            vars: &vars,
            home: home.map(Path::to_owned),
        };
        let mut out = Vec::new();
        let result = forget_key(&env, &RealFs, &mut out);
        (result, String::from_utf8(out).unwrap())
    }

    fn stored(home: &Path) -> Result<(String, KeySource), ConfigError> {
        let vars = vars(&[]);
        let env = Env {
            vars: &vars,
            home: Some(home.to_owned()),
        };
        find_key(&env, &RealFs).map(|(key, source)| (key.expose().to_owned(), source))
    }

    fn key_path(home: &Path) -> PathBuf {
        home.join(".config").join("jag").join("config.yml")
    }

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    // CRITERION-01
    #[test]
    fn setup_stores_the_key_for_later_searches() {
        let home = tempfile::tempdir().unwrap();
        let run = run_setup(&[], Some(home.path()), Fake::secret(SENTINEL_KEY), true);
        run.result.unwrap();
        assert!(run.out.contains("Stored the key in"));
        assert!(!run.out.contains("SENTINEL"));
        let path = key_path(home.path());
        assert_eq!(mode(&path), 0o600);
        assert_eq!(mode(path.parent().unwrap()), 0o700);
        let (text, source) = stored(home.path()).unwrap();
        assert_eq!(text, SENTINEL_KEY);
        assert_eq!(source, KeySource::ConfigFile(path));
    }

    #[test]
    fn setup_trims_the_typed_key() {
        let home = tempfile::tempdir().unwrap();
        run_setup(&[], Some(home.path()), Fake::secret("  abc123  \n"), true)
            .result
            .unwrap();
        assert_eq!(stored(home.path()).unwrap().0, "abc123");
    }

    // CRITERION-72 (the unit part)
    #[test]
    fn setup_with_no_terminal_stores_nothing() {
        let home = tempfile::tempdir().unwrap();
        let run = run_setup(&[], Some(home.path()), Fake::default(), false);
        assert_eq!(run.result.unwrap_err(), ConfigError::NoTerminal);
        assert_eq!(run.calls, 0);
        assert!(run.out.is_empty());
        assert!(!home.path().join(".config").exists());
        let failure = Failure::from(ConfigError::NoTerminal);
        assert_eq!(failure.exit_code, 2);
        assert!(failure.render().contains(JAG_VARIABLE_NAME));
    }

    const JAG_VARIABLE_NAME: &str = "JAG_TYPESAFE_API_KEY";

    #[test]
    fn setup_with_no_home_writes_nothing() {
        let run = run_setup(&[], None, Fake::default(), true);
        assert_eq!(run.result.unwrap_err(), ConfigError::NoHome);
        assert_eq!(run.calls, 0);
    }

    // CRITERION-03
    #[test]
    fn setup_asks_before_it_replaces_a_key() {
        let old = "config:\n  api-key: old-key\n";
        // The prompter maps `n`, an empty answer, and end of input to `false`
        // (see `the_answer_parser_accepts_only_y_and_yes`).
        let home = tempfile::tempdir().unwrap();
        let path = write_file(home.path(), old, 0o600);
        let run = run_setup(
            &[],
            Some(home.path()),
            Fake::answering(false, "new-key"),
            true,
        );
        run.result.unwrap();
        assert!(run.out.contains("Kept the stored key."));
        assert_eq!(std::fs::read_to_string(path).unwrap(), old);

        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), old, 0o600);
        let run = run_setup(
            &[],
            Some(home.path()),
            Fake::answering(true, "new-key"),
            true,
        );
        run.result.unwrap();
        assert_eq!(stored(home.path()).unwrap().0, "new-key");
    }

    #[test]
    fn the_answer_parser_accepts_only_y_and_yes() {
        for yes in ["y", "Y", "yes", "YES", "Yes\n", " y \r\n"] {
            assert!(is_yes(yes), "{yes:?}");
        }
        for no in ["", "\n", "n", "no", "yep", "ye", "yy", "1"] {
            assert!(!is_yes(no), "{no:?}");
        }
    }

    #[test]
    fn a_file_with_no_key_asks_no_question() {
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "# nothing\n", 0o600);
        let run = run_setup(&[], Some(home.path()), Fake::secret("k1"), true);
        run.result.unwrap();
        assert_eq!(stored(home.path()).unwrap().0, "k1");
    }

    #[test]
    fn setup_refuses_a_typed_key_with_a_bad_format() {
        let home = tempfile::tempdir().unwrap();
        let run = run_setup(
            &[],
            Some(home.path()),
            Fake::secret("jag test SENTINEL"),
            true,
        );
        let error = run.result.unwrap_err();
        assert_eq!(error, ConfigError::KeyTypedBad);
        let failure = Failure::from(error);
        assert_eq!(failure.exit_code, 2);
        assert!(!failure.render().contains("SENTINEL"));
        assert!(!run.out.contains("SENTINEL"));
        assert!(!key_path(home.path()).exists());
    }

    #[test]
    fn setup_stops_on_an_open_key_file_before_any_question() {
        let home = tempfile::tempdir().unwrap();
        let old = "config:\n  api-key: old-key\n";
        let path = write_file(home.path(), old, 0o644);
        let run = run_setup(&[], Some(home.path()), Fake::default(), true);
        let error = run.result.unwrap_err();
        assert!(matches!(error, ConfigError::KeyFileOpen { .. }));
        assert_eq!(Failure::from(error).exit_code, 3);
        assert_eq!(run.calls, 0);
        assert_eq!(std::fs::read_to_string(path).unwrap(), old);
    }

    #[test]
    fn setup_and_forget_leave_a_bad_yaml_file_as_it_is() {
        let home = tempfile::tempdir().unwrap();
        let text = "config:\n  api-key: [SENTINEL\n";
        let path = write_file(home.path(), text, 0o600);
        let run = run_setup(&[], Some(home.path()), Fake::default(), true);
        assert!(matches!(
            run.result.unwrap_err(),
            ConfigError::KeyFileBad { .. }
        ));
        let (result, _) = run_forget(&[], Some(home.path()));
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::KeyFileBad { .. }
        ));
        assert_eq!(std::fs::read_to_string(path).unwrap(), text);
    }

    #[test]
    fn setup_and_forget_give_exit_3_on_an_unreadable_file() {
        if is_root() {
            return;
        }
        let home = tempfile::tempdir().unwrap();
        write_file(home.path(), "config:\n  api-key: k\n", 0o000);
        let run = run_setup(&[], Some(home.path()), Fake::default(), true);
        assert!(matches!(
            run.result.unwrap_err(),
            ConfigError::KeyFileUnreadable { .. }
        ));
        let (result, _) = run_forget(&[], Some(home.path()));
        let failure = Failure::from(result.unwrap_err());
        assert_eq!(failure.exit_code, 3);
    }

    #[test]
    fn setup_and_forget_refuse_a_file_with_other_keys_and_a_link() {
        use std::os::unix::fs::symlink;
        let home = tempfile::tempdir().unwrap();
        let text = "config:\n  api-key: k\nother: 1\n";
        let path = write_file(home.path(), text, 0o600);
        let run = run_setup(&[], Some(home.path()), Fake::default(), true);
        assert!(matches!(
            run.result.unwrap_err(),
            ConfigError::KeyFileHasOtherKeys { .. }
        ));
        let (result, _) = run_forget(&[], Some(home.path()));
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::KeyFileHasOtherKeys { .. }
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);

        // A link to a private file: both refuse, and the link and the target stay.
        let link_home = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();
        let target = target_dir.path().join("real.yml");
        std::fs::write(&target, "config:\n  api-key: k\n").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();
        let dir = link_home.path().join(".config").join("jag");
        std::fs::create_dir_all(&dir).unwrap();
        symlink(&target, dir.join("config.yml")).unwrap();
        let run = run_setup(&[], Some(link_home.path()), Fake::default(), true);
        assert!(matches!(
            run.result.unwrap_err(),
            ConfigError::KeyFileIsLink { .. }
        ));
        let (result, _) = run_forget(&[], Some(link_home.path()));
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::KeyFileIsLink { .. }
        ));
        assert!(
            std::fs::symlink_metadata(dir.join("config.yml"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(target.exists());
        assert_eq!(stored(link_home.path()).unwrap().0, "k");
    }

    #[test]
    fn setup_notes_a_variable_that_wins_over_the_file() {
        let home = tempfile::tempdir().unwrap();
        let run = run_setup(
            &[("TYPESAFE_API_KEY", "from-variable")],
            Some(home.path()),
            Fake::secret("k"),
            true,
        );
        run.result.unwrap();
        assert!(
            run.out
                .contains("Note: TYPESAFE_API_KEY is set, and it wins over the stored key.")
        );
        assert!(!run.out.contains("from-variable"));
    }

    // CRITERION-04
    #[test]
    fn forget_removes_the_key_file() {
        let home = tempfile::tempdir().unwrap();
        let path = write_file(home.path(), "config:\n  api-key: k\n", 0o600);
        let (result, out) = run_forget(&[], Some(home.path()));
        result.unwrap();
        assert!(out.contains("Removed the stored key from"));
        assert!(!path.exists());
        assert_eq!(stored(home.path()).unwrap_err(), ConfigError::KeyMissing);
    }

    #[test]
    fn forget_with_no_key_exits_ok_two_times() {
        let home = tempfile::tempdir().unwrap();
        for _ in 0..2 {
            let (result, out) = run_forget(&[], Some(home.path()));
            result.unwrap();
            assert!(out.contains("No stored key in"));
        }
        write_file(home.path(), "# nothing\n", 0o600);
        let (result, out) = run_forget(&[], Some(home.path()));
        result.unwrap();
        assert!(out.contains("No stored key in"));
        assert!(key_path(home.path()).exists());
    }

    #[test]
    fn forget_with_no_home_exits_ok() {
        let (result, out) = run_forget(&[], None);
        result.unwrap();
        assert!(out.contains("HOME is not set"));
    }

    #[test]
    fn forget_does_not_check_the_mode_and_notes_a_variable() {
        let home = tempfile::tempdir().unwrap();
        let path = write_file(home.path(), "config:\n  api-key: k\n", 0o644);
        let (result, out) = run_forget(&[(JAG_VARIABLE_NAME, "v")], Some(home.path()));
        result.unwrap();
        assert!(!path.exists());
        assert!(out.contains("Note: JAG_TYPESAFE_API_KEY is still set, so jag still has a key."));
    }

    #[test]
    fn the_output_of_the_success_paths_never_holds_the_key() {
        let home = tempfile::tempdir().unwrap();
        let run = run_setup(&[], Some(home.path()), Fake::secret(SENTINEL_KEY), true);
        run.result.unwrap();
        assert!(!run.out.contains("SENTINEL"));
        let run = run_setup(&[], Some(home.path()), Fake::answering(false, "x"), true);
        run.result.unwrap();
        assert!(run.out.contains("Kept the stored key."));
        assert!(!run.out.contains("SENTINEL"));
        let (result, out) = run_forget(&[], Some(home.path()));
        result.unwrap();
        assert!(!out.contains("SENTINEL"));
    }
}
