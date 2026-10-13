---
title: "Store a key one time: project skeleton, key lookup, and key store"
feature: FEATURE-01
stories: [STORY-01, STORY-02, STORY-03, STORY-04, STORY-05]
kind: build
lifecycle_state: draft
inherits: [QS-5, IF-3, IF-4, C-cli, C-config, CV-1, CV-3, CV-6, CV-7]
session: 21508ce9-788b-40ed-be8d-03797016b20b
---

# Store a key one time: project skeleton, key lookup, and key store

## Problem

The user stores a TypeSafe key one time. jag finds the key on each later
search, never shows it, and tells a new user where to get one (FEATURE-01,
CAPABILITY-01).

- STORY-01: an agent builder stores the key with one setup run, so that each
  later search finds the key with no more steps. A TypeSafe key is enough to
  start (CONSTRAINT-03).
- STORY-02: an operator overrides the stored key for one shell or one job.
  jag reads `JAG_TYPESAFE_API_KEY`, then `TYPESAFE_API_KEY`, then
  `~/.config/jag/config.yml` (DECISION-03).
- STORY-03: setup asks before it replaces a key, and one command removes the
  key (DECISION-08).
- STORY-04: jag never shows the key, and it refuses a key file that other
  users can read (CONSTRAINT-02).
- STORY-05: the missing-key message gives the page where the user gets a key
  (DECISION-09).

The repository holds no code yet. This spec also creates the Rust project,
its tests, its lint, and its CI. Each later feature builds on this project.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Language and program form | Rust, one program. This spec builds the binary `jag`. `jag-bench` comes later | 0005-build-jag-as-one-rust-binary.md |
| 2 | Project form | One Cargo package, `jag-grep` on crates.io, with the command `jag`: a library (`src/lib.rs`) with one module for each `C-` component, and a thin binary (`src/main.rs`) | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md |
| 3 | Toolchain and lint | Rust 2024 edition. `rust-toolchain.toml` pins one stable version with `rustfmt` and `clippy`. `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must pass | 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md |
| 4 | CI | GitHub Actions on each pull request and on each push to `main`: fmt, clippy, and test, on `ubuntu-latest` and `macos-latest` | system-design.md deployment-3; 0011-build-jag-as-one-cargo-package-with-one-module-per-component.md |
| 5 | Third-party crates | `clap` 4 (derive), `serde` (derive), `serde-saphyr` (YAML), `rpassword` (no-echo prompt). Tests: `assert_cmd`, `predicates`, `tempfile` | 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md |
| 6 | Options for setup and removal | `--setup` stores the key. `--forget-key` removes it | 0006-run-setup-the-readiness-check-and-key-removal-as-options.md |
| 7 | Key sources and order | `JAG_TYPESAFE_API_KEY`, then `TYPESAFE_API_KEY`, then `~/.config/jag/config.yml`. The first key found wins | DECISION-03 |
| 8 | Config file form | A nested map: `config:`, then `api-key: <key>` under it | human:404pilo, 2026-10-13 (Q-b) |
| 9 | Exit codes | 3: each local key problem that jag finds before a call. These are: no key, a key file that others can read, a key file that jag cannot read, a key file that is not valid YAML, a key with a bad format in a variable or in the file, and a failed write of the key file (`KeyFileWrite`). 4 stays for a key that the service refuses. 2: a wrong use of `--setup` or `--forget-key`. These are: no terminal, a typed key with a bad format, no `$HOME`, a key file with other keys, and a key file that is a symbolic link | 0007-number-the-exit-codes-above-2-from-3-to-8.md; CV-6; 0013-exit-2-when-setup-runs-with-no-terminal.md |
| 10 | Key secrecy | Only the `config` module reads the key. The `ApiKey` type never prints itself. No option takes the key. `--setup` reads it with no echo. jag refuses a key file with any group or other permission bit | system-design.md CV-6 |
| 11 | Failure shape | Each failure has a cause, a fix, and an exit code. In text mode, jag writes it to the error stream | system-design.md CV-1, CV-3 |
| 12 | Missing-key page | `https://console.typesafe.ai/keys` | DECISION-09; TypeSafe quick start, `docs.typesafe.ai/introduction/quickstart.md` |
| 13 | Setup tests the key with the service | No. Setup stores the key only. `--check` (FEATURE-02) tests it | human:404pilo, 2026-10-13 (Q-d) |
| 14 | Empty variable | An empty or blank variable counts as not set | The earlier prototype, its config module (inferred) |
| 15 | Bad source | A source that is set but bad is an error. jag does not go on to the next source | The earlier prototype, its config module (inferred) |
| 16 | Key format | After trim: one or more printable ASCII characters (`0x21` to `0x7E`), no space | The earlier prototype, its config module (inferred) |
| 17 | Test seams | `config` takes the environment, the home folder, and the file system as arguments, so tests give fakes | The earlier prototype, its config module (inferred) |
| 18 | Other keys in the config file | Release 1 writes only `config.api-key`. A search ignores other keys. `--setup` and `--forget-key` refuse to change a file that holds other keys (exit 2), so jag never deletes text that the user wrote | system-design.md DF-1 |
| 19 | YAML error text | jag never writes the text of a `serde_saphyr::Error`, with or without a snippet, because serde errors can quote a value. jag writes only the path and the line from `Error::location()` | QS-5, CV-6; serde-saphyr docs (`Error::location`) |
| 20 | Symbolic links | A search follows a link to the key file and checks the mode of the open file. `--setup` and `--forget-key` refuse a `config.yml` that is a link (exit 2), because a rename replaces the link and a removal leaves the target | CV-6 |

## Design

### Project skeleton

Files that this spec adds:

```
Cargo.toml
Cargo.lock
rust-toolchain.toml
rustfmt.toml            (empty file: default style)
.github/workflows/ci.yml
src/main.rs             thin: calls jag::cli::run(), exits with its code
src/lib.rs              pub mod cli; pub mod config; pub mod failure;
src/cli.rs              C-cli: option parsing, dispatch, exit code
src/config.rs           C-config: ApiKey, lookup, store, removal
src/config/             private submodules if config.rs passes 300 lines
src/failure.rs          Failure { cause, fix, exit_code }
tests/setup.rs          end-to-end tests of --setup and --forget-key
tests/secrecy.rs        the key never shows (CRITERION-05)
```

- `Cargo.toml`: `name = "jag-grep"`, the crates.io package name. The
  crates.io name `jag` belongs to another crate. The command stays `jag`, with
  `[[bin]] name = "jag"` and `path = "src/main.rs"`. The library keeps
  `[lib] name = "jag"`, so that code uses `jag::config`. Also set
  `edition = "2024"`, `license = "MIT"`,
  `repository = "https://github.com/ai-ready-future/jag"`,
  `rust-version` equal to the pinned toolchain. Set
  `[lints.clippy] all = "warn"` and `[lints.rust] unsafe_code = "forbid"`.
- Later features add one module for each `C-` component (`search`,
  `prerank`, `scorer`, `output`, `check`) to `src/lib.rs`. Each module keeps
  its own unit tests in a `#[cfg(test)] mod tests`.
- `src/main.rs` holds no logic. `cli::run` returns an `ExitCode`.
- `.gitignore` already holds Python lines. Add `/target`.
- CI (`.github/workflows/ci.yml`): one job, matrix `os: [ubuntu-latest,
  macos-latest]`. Steps: checkout, install the toolchain from
  `rust-toolchain.toml`, cache, `cargo fmt --check`,
  `cargo clippy --locked -- -D warnings`,
  `cargo test --locked`. Keep the existing `adr-gate.yml` as it is.

### Command line (C-cli)

`clap` derive parses the options. This spec adds two options to the
`jag` command line (IF-3):

- `jag --setup`
- `jag --forget-key`

Each is exclusive: with a term, a path, or the other option, clap gives a
usage error, exit 2. `jag --help` and `jag --version` work. A run with no
option and no term prints the usage and exits with 2. The search form
`jag TERM "QUESTION" [PATH...]` comes with FEATURE-04.

`cli::run` maps each `Failure` to its exit code and writes
`jag: <cause>` and then `<fix>` on the error stream (CV-1, CV-3).

### Failure type

```rust
pub struct Failure {
    pub cause: String,   // never holds the key
    pub fix: String,     // one line the user can act on
    pub exit_code: u8,   // a code of IF-4
}
```

`config` returns `Result<_, ConfigError>`. `ConfigError` is an enum with a
variant for each cause, and `impl From<ConfigError> for Failure` gives the
text and the code. FEATURE-03 and FEATURE-11 reuse this type.

### Key type

```rust
pub struct ApiKey(String);   // field is private
```

- `Debug` and `Display` write `ApiKey(***)` and `***`.
- No `Serialize`, no `Clone`, and no `Deref` to
  `str`.
- One method, `pub(crate) fn expose(&self) -> &str`, gives the text. Only the
  model client of FEATURE-04 calls it, to build the `Authorization` header.
- `ApiKey::parse(raw: &str) -> Result<ApiKey, KeyFormatError>`: trims the
  text, then checks decision 16. The error never holds the text.

### Key lookup

```rust
pub enum KeySource { JagVariable, TypeSafeVariable, ConfigFile(PathBuf) }

pub struct Env<'a> { pub vars: &'a dyn Fn(&str) -> Option<String>, pub home: Option<PathBuf> }

pub fn find_key(env: &Env, fs: &dyn Fs) -> Result<(ApiKey, KeySource), ConfigError>;
```

`Fs` is a small trait over the file system: open and read a file with the
mode of the open handle, check if a path is a symbolic link (no follow),
write a private file, remove a file, and create a private folder.
`RealFs` is the only production impl. Tests use `tempfile` folders with
`RealFs`, or a fake.

Order (decision 7):

1. `JAG_TYPESAFE_API_KEY`. Blank counts as not set (decision 14).
2. `TYPESAFE_API_KEY`. Same rule.
3. The file `<home>/.config/jag/config.yml`. `home` comes from `$HOME`.
   With no `$HOME`, or an empty one, skip this step.
   - Open the path with `File::open`. This follows a symbolic link
     (decision 20). `NotFound`: go to step 4. Another open error:
     `KeyFileUnreadable`, exit 3, fix "Check the permissions of <path>".
   - Check the mode with `metadata()` of the open handle (`fstat`), before
     jag reads a byte. When `mode & 0o077 != 0`, return `KeyFileOpen`,
     exit 3, fix `chmod 600 <path>` (CRITERION-06). Read the text from the
     same handle, so no other file can take its place between the check and
     the read.
   - A read error: `KeyFileUnreadable`, exit 3.
   - Parse the text into `struct File { config: Option<Section> }`,
     `struct Section { #[serde(rename = "api-key")] api_key: Option<String> }`.
     A search ignores other keys (decision 18).
   - A YAML error: `KeyFileBad`, exit 3. The cause is
     "<path> is not valid YAML (line N)". N comes from
     `Error::location()`. When there is no location, leave out the line.
     jag never writes the error text (decision 19).
   - No `config.api-key`, or a blank value: go to step 4.
   - A value with a bad format: `KeyFormatBad { source }`, exit 3.
4. No source: `KeyMissing`, exit 3 (CRITERION-07).

A source that is set but bad stops the lookup with its error
(decision 15). The error names the source, for example "the variable
JAG_TYPESAFE_API_KEY", and never the value. A bad format in a variable gives
`KeyFormatBad`, exit 3, fix "Set <VARIABLE> to the key from
https://console.typesafe.ai/keys, or unset it".

The missing-key text:

```
jag: no TypeSafe API key found.
Get a key at https://console.typesafe.ai/keys, then run `jag --setup`,
or set JAG_TYPESAFE_API_KEY.
```

### Load the file for a change

`--setup` and `--forget-key` share one function, `load_for_change`. It
returns one of: no file, a file with no key, a file with a key, or an
error. It does these checks in this order. Each error stores nothing and
changes nothing:

1. No `$HOME`, or an empty one: `NoHome`, exit 2. Cause "HOME is not set,
   so jag has no folder for the key file". Fix "Set HOME, or set
   JAG_TYPESAFE_API_KEY". (`--forget-key` handles this case in its own
   step 1.)
2. `symlink_metadata(path)` shows a symbolic link: `KeyFileIsLink`, exit 2.
   Fix "Edit the target of <path> by hand, or replace the link with a
   file" (decision 20).
3. Open the file and read it from the same handle, as in key lookup step 3.
   `NotFound` gives "no file". Another open or read error gives
   `KeyFileUnreadable`, exit 3. Keep the mode of the open handle in the
   result for the caller.
4. Parse two times. serde-saphyr gives an unknown field and many syntax
   errors as the same `Message` error kind, so jag cannot tell them apart
   by the kind or by the text.
   - First, parse with the loose structs of key lookup step 3. An error:
     `KeyFileBad`, exit 3, with the path and the line only (decision 19).
   - Then, parse the same text with a strict copy of the structs, which
     have `#[serde(deny_unknown_fields)]` on `File` and on `Section`. The
     loose parse passed, so an error here means an unknown key:
     `KeyFileHasOtherKeys`, exit 2. Cause "<path> holds settings other than
     config.api-key". Fix "Remove the other settings, or edit the file by
     hand".

`load_for_change` does not check the mode. `--setup` checks it in its own
step 3.

### Write the file

`write_key_file(home, key)`:

1. Create `<home>/.config` with `fs::create_dir_all` and the default mode,
   when it does not exist. Never change the mode of `<home>/.config`.
2. Create `<home>/.config/jag` with `DirBuilder` and mode `0o700`, when it
   does not exist. Do not change the mode of a folder that exists.
3. Create `<home>/.config/jag/config.yml.tmp-<pid>` with
   `OpenOptions::create_new(true)` and mode `0o600`. Write
   `config:\n  api-key: <key>\n` (decision 8), then `sync_all`.
4. Rename the temporary file over `config.yml`.
5. When step 3 or 4 fails after the temporary file exists, remove the
   temporary file, then return `KeyFileWrite`, exit 3, with the path and the
   `io::ErrorKind` only.

### Setup (`--setup`)

The setup function takes a `Prompter`, the environment, the file system,
and a writer for its messages, so a unit test captures all output.

1. When standard input is not a terminal (`std::io::IsTerminal`), write
   "jag: --setup needs a terminal to read the key. To give the key with no
   terminal, set JAG_TYPESAFE_API_KEY." to the error stream, store nothing,
   and exit 2 (CRITERION-72, 0013-exit-2-when-setup-runs-with-no-terminal.md).
2. `load_for_change`. On an error, write the failure and exit with its code.
3. When the file exists and its mode has a group or other bit, stop before
   the question: `KeyFileOpen`, exit 3, fix `chmod 600 <path>`. Setup does
   not ask and does not keep an open file, so a later search cannot fail
   with exit 3 after a setup that exited 0.
4. When the file holds a key, ask
   "A key is already stored in <path>. Replace it? [y/N] ". Only `y` or
   `yes` (any case) goes on. Each other answer, and end of input, keeps the
   file as it is, prints "Kept the stored key.", and exits 0 (CRITERION-03).
5. Read the key with `rpassword::prompt_password("TypeSafe API key: ")`. The
   terminal shows no echo (CV-6).
6. `ApiKey::parse`. On an error, print "The key has a space or a character
   that is not plain ASCII." with no key text, store nothing, and exit 2.
7. `write_key_file`.
8. Print "Stored the key in <path>." and exit 0. When `JAG_TYPESAFE_API_KEY`
   or `TYPESAFE_API_KEY` is set, also print "Note: <VARIABLE> is set, and it
   wins over the stored key."

Setup makes no network call (decision 13).

### Key removal (`--forget-key`)

1. No `$HOME`, or an empty one: print "No stored key: HOME is not set." and
   exit 0. A search also has no file source in this case.
2. `load_for_change`. On an error, write the failure and exit with its code.
   A bad YAML file or an unreadable file gives exit 3. A link or a file with
   other keys gives exit 2. The file stays as it is.
3. No file, or a file with no `config.api-key`: print
   "No stored key in <path>." and exit 0.
4. Delete `config.yml`. Decision 18 makes sure that the file holds only the
   key.
5. Print "Removed the stored key from <path>." and exit 0.
6. When a key variable is set, also print "Note: <VARIABLE> is still set,
   so jag still has a key."

`--forget-key` does not check the file mode. It removes the file and shows
no key text, so an open mode does not stop it.

### Acceptance criteria and their tests

| Criterion | Test |
|---|---|
| CRITERION-01 Setup stores the key for later searches | Unit: `setup` with a fake prompter and a `tempfile` home writes `config.yml` with mode `0o600`; then `find_key` with no variables returns the key and `KeySource::ConfigFile`. The folder has mode `0o700`. |
| CRITERION-02 jag finds the key in a fixed order | Unit, table test over the 8 combinations of the 3 sources: the first set source wins. Blank variables count as not set. |
| CRITERION-03 Setup asks before it replaces a key | Unit with a fake prompter: answer `n`, empty, and end of input keep the old file byte for byte; answer `y` replaces the key. |
| CRITERION-04 One command removes the stored key | `tests/setup.rs`: write a file into a temp `HOME`, run `jag --forget-key` with `assert_cmd`, check exit 0 and that `find_key` gives `KeyMissing`. Also the case with no file: exit 0 and the "No stored key" text. |
| CRITERION-05 jag never shows the key | The key text holds `SENTINEL` in each case, for example `jag-test-SENTINEL-4f9c`. `tests/secrecy.rs` (binary): setup with no terminal, `--forget-key` with a key file, `--forget-key` on a bad YAML file and on a file with other keys, `--setup` on a key file with mode `0o644`, and `--help`. Unit tests (all output goes to a captured writer): `find_key` with a bad key format in each variable (`jag test SENTINEL` with a space) and in the file; a YAML error on the `api-key` line itself (`config:\n  api-key: [SENTINEL`); the setup success path and the "Kept the stored key" path with a fake prompter. For each case, the failure text, the cause, the fix, stdout, and stderr never hold `SENTINEL`. Also `format!("{:?}", key)` and `format!("{}", key)` do not hold the text. |
| No `$HOME` | Unit: `--setup` with no home gives `NoHome`, exit 2, and writes nothing. `--forget-key` with no home exits 0 with "HOME is not set". `find_key` with no home and no variable gives `KeyMissing`. |
| Bad or unreadable file in a change | Unit: `--setup` and `--forget-key` on a bad YAML file give exit 3 and leave the file byte for byte. On a file with mode `0o000`, both give `KeyFileUnreadable`, exit 3. Skip this case when the tests run as root. |
| Open file in setup | Unit: `--setup` on a key file with mode `0o644` gives exit 3 and the `chmod 600` fix before any question. The fake prompter records no call. |
| Other keys and links | Unit: a file with `config:\n  api-key: k\nother: 1` gives exit 2 for `--setup` and `--forget-key`, and a search still finds `k`. A `config.yml` that is a link to a `0o600` file: a search finds the key; `--setup` and `--forget-key` give exit 2 and leave the link and the target. A link to a `0o644` file: a search gives `KeyFileOpen`. |
| Write failure | Unit with a fake `Fs` whose rename fails: `write_key_file` gives `KeyFileWrite`, exit 3, and no `config.yml.tmp-*` file stays. A real `<home>/.config` that setup creates keeps the default mode, not `0o700`. |
| CRITERION-06 jag refuses a key file that others can read | Unit: a file with mode `0o640`, `0o604`, and `0o644` gives `KeyFileOpen`, exit 3, and the fix `chmod 600 <path>`. Mode `0o600` and `0o400` pass. |
| CRITERION-07 A missing key points to where to get one | Unit: `Failure::from(ConfigError::KeyMissing)` has exit 3, and its text holds `https://console.typesafe.ai/keys` and `jag --setup`. |
| CRITERION-08 A TypeSafe key is enough to start | Unit: with only `TYPESAFE_API_KEY` set, and no other variable or file, `find_key` returns the key. Setup asks for no value other than the key. The run of a search is in FEATURE-04. |
| CRITERION-72 Setup with no terminal fails clearly | `tests/setup.rs`: run `jag --setup` with stdin from a pipe and a temp `HOME`. Check exit 2, a message on stderr, an empty stdout, and that no `config.yml` exists. |

Each `assert_cmd` test sets `HOME` to a `tempfile` folder and calls
`env_remove("JAG_TYPESAFE_API_KEY")` and `env_remove("TYPESAFE_API_KEY")`,
so a key variable on the test machine does not change the result. Unit
tests give the environment as a fake (decision 17) and never read the real
environment.

The interactive path of `--setup` needs a terminal, so its unit tests use a
`Prompter` trait (`confirm(&str) -> io::Result<bool>`,
`secret(&str) -> io::Result<String>`) with a fake. `TtyPrompter` is the
production impl over `rpassword` and standard input.

## Build order

1. Skeleton: `Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`,
   `src/lib.rs`, `src/cli.rs` with `--help` and `--version`, CI. Prove that
   fmt, clippy, and test pass locally and in CI.
2. `failure.rs` and `ApiKey` with the secrecy unit tests (STORY-04, part 1).
3. `find_key` with the order table, the `fstat` mode check, and the YAML
   error text (STORY-02, STORY-04 part 2, STORY-05).
4. `load_for_change` and `write_key_file` with their failure tests.
5. `--setup` with the `Prompter` trait (STORY-01, CRITERION-72).
6. Replace confirmation and `--forget-key` (STORY-03).
7. `tests/secrecy.rs` over every path (STORY-04, CRITERION-05).

## Assumptions

- Decisions 14 to 17 come from the config module of the earlier TypeScript
  prototype. The ADRs win over that code:
  jag does not read `JAG_API_KEY`, `JAG_API_KEY_FILE`, or
  `~/.config/jag/.key`.
- `~` means `$HOME` on Linux and macOS. jag does not read
  `XDG_CONFIG_HOME` and does not use `~/Library` on macOS, because
  DECISION-03 names `~/.config/jag/config.yml`. With no `$HOME`, the file
  source is skipped.
- Setup creates `~/.config/jag/` with mode `0o700`, and `~/.config` with
  the default mode. It does not change the mode of a folder that exists.
- Exit 3 covers each local key problem, and exit 2 covers each wrong use of
  `--setup` or `--forget-key` (decision 9). These use the codes of ADR-0007
  and ADR-0013 and add no code.
- A replace (answer `y` in setup step 4) writes a new file, so it drops
  each YAML comment of the old file. Decision 18 keeps other keys out of
  the file, so only comments can go. This needs no code change.
- A user who wants other settings in `config.yml` must wait for DF-1. Until
  then, `--setup` and `--forget-key` refuse such a file and a search
  ignores the other keys (decision 18).
- `--forget-key` with no stored key exits 0, so a clean-up script can run it
  two times.
- `--setup` and `--forget-key` print a note when a key variable is set,
  because the variable wins over the file.

## Risks

- A later module logs the key through `expose()`. Signal: a `grep` for
  `expose(` finds a call outside the model client. Mitigation: the method is
  `pub(crate)`, and the secrecy test grows with each new error path.
- The no-echo prompt does not work the same on each terminal. Signal: a
  manual test of `jag --setup` on Linux or macOS shows the key as the user
  types it. Mitigation: setup step 1 refuses a run with no terminal, and the
  release check list has one manual test on each platform.
- The DECISION-03 text "at `config.api-key`" has two readings. Signal: a
  user writes `api-key:` at the top level and jag finds no key. Mitigation:
  the README shows the file form.
- Windows has no file mode. Signal: a Windows build request. Release 1
  supports Linux and macOS only (deployment-2).

## Out of scope

- The search command `jag TERM "QUESTION" [PATH...]` and the model client:
  FEATURE-04. It calls `find_key` and shows the missing-key failure.
- `--check`, which tests the key with the service: FEATURE-02.
- Model failures, credit, and service down (exit 4 to 6): FEATURE-03.
- Structured errors in JSON: FEATURE-11.
- Plain grep with no key: FEATURE-12.
- `jag-bench`: the feature for CAPABILITY-12.
- The CI scan that fails on private data (CONSTRAINT-01, deployment-3): no
  feature owns it yet.
- Release builds and publishing (ADR-0010): no feature owns them yet.
