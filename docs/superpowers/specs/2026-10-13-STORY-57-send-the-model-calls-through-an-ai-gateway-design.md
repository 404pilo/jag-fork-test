---
title: "Send the model calls through an AI gateway: the base URL and the model name from an option, a variable, or the config file"
feature: FEATURE-21
stories: [STORY-57, STORY-58]
kind: build
gh_issue: 119
lifecycle_state: draft
inherits: [QS-5, IF-1, IF-2, IF-4, C-cli, C-config, C-scorer, C-check, C-gateway, CV-1, CV-6, CV-7, DF-1]
session: 84086903-189a-4495-bbe8-0cfb867635cd
---

# Send the model calls through an AI gateway: the base URL and the model name from an option, a variable, or the config file

## Problem

The agent builder gives jag the base URL and the model name of an AI
gateway, so that jag uses the gateway that the team already uses
(STORY-57). The operator sets the two values as an option, a variable, or
in the config file, so that a value applies to one search, one job, or each
search (STORY-58). With no setting, jag calls TypeSafe directly and needs
only the TypeSafe key (CONSTRAINT-03).

The acceptance criteria:

- CRITERION-94: when the user sets a base URL and a model name, jag sends
  the same model calls to that address with that model name.
- CRITERION-95: when the user sets no base URL and no model name, jag calls
  TypeSafe directly and needs only the TypeSafe key.
- CRITERION-96: for the base URL and the model name, the command option
  wins over the variable, the variable wins over the config file, and the
  config file wins over the default.

The variables `JAG_BASE_URL` and `JAG_MODEL` exist (FEATURE-04 spec
decision 9). This spec adds the two options, the two keys of the config
file, the address forms of the TypeSafe docs, and failure texts that are
correct for a gateway.

Each gateway takes its own key, and a TypeSafe key must stay inside the
trust boundary of the user (QS-5, CV-6). So this spec also adds a separate
key for a gateway: `JAG_GATEWAY_API_KEY` and `config.gateway-api-key`
(0017-use-a-separate-key-for-an-ai-gateway.md).

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Spec grouping | One spec for STORY-57 and STORY-58. They share the code of the settings, and STORY-58 gives the sources of the two values of STORY-57 | docs/adlc.md (one feature gives one spec) |
| 2 | Options | `--base-url URL` and `--model NAME` | Inferred: the variable names of FEATURE-04 spec decision 9, and the pairs `--timeout` and `JAG_TIMEOUT`, `--retry-wait` and `JAG_RETRY_WAIT`, and `--max-scored` and `JAG_MAX_SCORED` (src/cli.rs) |
| 3 | Variables | `JAG_BASE_URL` and `JAG_MODEL`, as they are | FEATURE-04 spec decision 9 |
| 4 | Config file keys | `config.base-url` and `config.model` in `~/.config/jag/config.yml`, next to `config.api-key`. jag writes no other setting to the file, except the gateway key of decision 11 | STORY-58; CRITERION-96; FEATURE-01 spec decision 8 (the nested form); DF-1 (its trigger fires for these two settings only); 0015-read-the-base-url-and-the-model-name-from-config-base-url.md (accepted: human:404pilo, 2026-10-14, hitl answer) |
| 5 | Order | For each of the two settings by itself: the option, then the variable, then the config file, then the default. The first source that is set wins. A source that is set but bad stops the lookup. A source that wins hides a bad value of a lower source of the same setting | CRITERION-96; CV-7; FEATURE-01 spec decision 15; FEATURE-03 spec decision 9 and its test "an option hides a bad variable of the same time" |
| 6 | Defaults | `https://api.typesafe.ai/v1` and `jev-1.13.0` | FEATURE-04 spec decision 9; 0009-pin-the-jev-model-version-in-each-release.md |
| 7 | Address form | When the path of the base URL does not end with `/v1`, jag adds `/v1`. The call goes to `<base URL>/systemone`. So `https://api.typesafe.ai` and `https://api.typesafe.ai/v1` give the same call | TypeSafe docs (`/typesafe-docs/typesafeai`): the root URL is `https://api.typesafe.ai`, the SDK calls `<root URL>/v1/systemone`, and each gateway page gives a root URL; FEATURE-04 spec decision 9 (the default ends with `/v1`); 0016-add-v1-to-a-base-url-whose-path-does-not-end-with-v1.md (accepted: human:404pilo, 2026-10-14, hitl answer) |
| 8 | Address checks | For each source: `https`, or `http` only for `127.0.0.1`, `localhost`, or `[::1]`; no user name (`@`). Also no `?` and no `#`, because jag adds a path at the end | FEATURE-04 spec decision 29; `?` and `#` (inferred: jag adds a path at the end of the base URL) |
| 9 | Blank values | A blank variable, or a blank or empty value in the file, counts as not set. A blank option gives exit 2. The same rule applies to the two sources of the gateway key | FEATURE-01 spec decision 14; FEATURE-03 spec decision 2 (an option value that is not valid gives exit 2) |
| 10 | Bad values | Exit 2. The text names the source (`--base-url`, `JAG_BASE_URL`, or `config.base-url in <path>`) and never shows the value | FEATURE-04 spec decisions 26 and 29; FEATURE-03 spec decision 10; CV-6 |
| 11 | The key with a gateway | On the gateway route, jag sends only a gateway key: `JAG_GATEWAY_API_KEY`, then `config.gateway-api-key`. The first key found wins. jag never sends a key from `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, or `config.api-key` to a gateway. With no gateway key, the search stops before the call with exit 3. On the direct route, jag uses the order of DECISION-03 and does not read the gateway key. jag never reads a key source of the other route, so a bad value there stops nothing | 0017-use-a-separate-key-for-an-ai-gateway.md (accepted: human:404pilo, 2026-10-14, hitl answer). DECISION-03 gives the key order of the direct route only, and 0017 gives the key order of the gateway route. QS-5; CV-6; CV-7; IF-2; TypeSafe docs (each gateway takes its own key as the API key of the SDK) |
| 12 | When jag reads the config file | At most one time in each run. The settings step (step 2 of the search) and the key step (step 3) share the read. jag reads the file only when one of the two settings, or the key of the route, has no source above the file | FEATURE-04 spec "The search flow"; DECISION-03; 0017 (the gateway key is in the same file); one read for the key and the settings (inferred: both come from the same file) |
| 13 | Mode check | Each read of the file checks the mode first, also a read for the settings only. A file with a group or other permission bit gives exit 3, as now | CV-6; FEATURE-01 spec decision 10; 0015 Consequences |
| 14 | File errors | The exit 3 and the texts of FEATURE-01 (not a regular file, cannot read, a mode that others can use, not valid YAML), whatever value jag looked for | FEATURE-01 spec decision 9 |
| 15 | `--setup` and `--forget-key` | They work on the key of the route. The route comes from `JAG_BASE_URL`, then `config.base-url`, then the default, because the two options take no `--base-url` (decision 19). On the direct route they store or remove `config.api-key`, as now. On the gateway route they store or remove `config.gateway-api-key`, and `--setup` says so before the prompt. They keep each other value of the file: the key of the other route, `config.base-url`, and `config.model`. `--forget-key` removes the file when no value is left. A file with another key still gives exit 2 | 0017 Consequences ("`--setup`, `--forget-key`, and `--check` must handle the gateway key. The parent spec gives how"); 0015 ("jag writes no other setting to the file, except the gateway key of ADR-0017"); FEATURE-01 spec decision 18 (jag never deletes a setting that the user wrote); the route of a search without the option (inferred: a user who names the gateway in a variable or in the file stores the key of that gateway) |
| 16 | Route | Direct when the base URL, after decision 7, uses `https`, has the host `api.typesafe.ai` in any case, has no port or port 443, and has the path `/v1`. Each other address is a gateway | Inferred: the TypeSafe docs give `https://api.typesafe.ai` as the address of the TypeSafe API, and each gateway has another host. The case of the host and port 443 do not change the address |
| 17 | Failure texts | The direct route keeps each cause text. The gateway route says "the gateway". A fix names the source of the setting to check. With no source, the fix tells the user to update jag | CV-1; FEATURE-03 spec decision 12 (one fix for each cause) |
| 18 | Key texts | The direct route keeps the key texts of FEATURE-01. The gateway route has its own texts: `no gateway key found.`, and fixes that name `JAG_GATEWAY_API_KEY` and `config.gateway-api-key`. A fix never tells the user to set a key variable of the other route | 0017 ("The error text names `JAG_GATEWAY_API_KEY`"); DECISION-03 (the direct route only); FEATURE-03 spec decision 12 (one fix for each cause); CV-1 |
| 19 | Where jag checks the options | In step 1, after `--max-scored` and after `--max-chars` or `--no-max-chars` (FEATURE-09): `--base-url`, then `--model`. They conflict with `--setup` and `--forget-key`, and not with `--check`. A search with no `--ask` checks them too, and a good value has no effect there | FEATURE-03 spec decisions 11 and 14; FEATURE-12 spec decision 5; FEATURE-13 spec decision 30; src/cli.rs (`search_args`) |
| 20 | `--check` | The same options, settings, address rule, route, and key of the route as a search. A bad value stops the check before the first item (exit 2). A file error of the settings read goes to item 1 (see "The readiness check"). The item texts stay | FEATURE-02 spec decision 17 and "Out of scope"; 0017 Consequences; the check shows what a search does (inferred); item 1 shows a file error of the settings read (inferred: with no route, jag cannot find the key, and FEATURE-02 shows a file error of the key read on item 1) |
| 21 | Structured output | With `--json`, a failure of this spec goes in the error document, with the same cause and fix. No new field | FEATURE-11 spec decisions 9 and 21 |
| 22 | Usage line | The search lines of FEATURE-12 do not change, because `[OPTIONS]` holds the two options. The check line gets `[--base-url URL] [--model NAME]`. The code that merges later keeps the lines of each merged feature | FEATURE-12 spec decision 22; 0018-take-the-question-only-from-the-ask-option.md; FEATURE-13 spec decision 31; FEATURE-11 spec "The option (C-cli)" (the same merge rule) |
| 23 | TLS and proxy | No new setting. The certificate store of the machine and the proxy variables, as now | 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md |
| 24 | Crates | No new crate | 0012-use-clap-serde-saphyr-and-rpassword-as-the-outside-crates.md; 0014-call-the-model-service-with-ureq-and-serde-json-on-std.md |
| 25 | Docs | The README gets the section "Use an AI gateway" | FEATURE-11 spec decision 29 (a README section for each feature that the user sees) (inferred) |
| 26 | Story state | `ready` in the spec PR: the spec is in the PR, FEATURE-04 (the only feature in `blocked_by`) is `done`, and the ASK has its answer (human:404pilo, 2026-10-14) | docs/adlc.md rule 11 |
| 27 | Order of the steps | Step 2 of a search finds the settings: the base URL, the model, and the times. So it finds the route. Step 3 finds the key of the route. The cost limit and the size limit come after the key, as now. So a bad setting and no key give exit 2, not 3. Steps 2 and 3 still come before the term and the scan. A search with no `--ask` reads no model setting and no key | 0017 Consequences ("jag must know the route before it looks for the key. So a search reads the settings before the key"); FEATURE-04 spec decision 18 (the key before the scan): 0017 changes its rule that only an argument error of step 1 comes before the key; FEATURE-14 spec decision 9 (the key and the settings come before the term and the scan); FEATURE-12 spec decision 4 |

## Design

### Files

```
src/config/model.rs   Origin, Route, SettingOptions, the address rule, the order
                      of the sources, and the values of the file
src/config.rs         ConfigFile (one read), the three new keys of Section and
                      StrictSection, KeySource and find_key for each route,
                      Env::set_key_variable, the new ConfigError texts, and
                      is_file_error
src/config/change.rs  FileValues, write_key_file with each value of the file,
                      and the escapes of yaml_scalar
src/config/setup.rs   --setup and --forget-key work on the key of the route,
                      and keep the other values of the file
src/cli.rs            --base-url, --model, the usage lines, the help of
                      --setup and --forget-key, the order of the steps, and
                      the flow of the check
src/check.rs          run takes the ConfigFile and the result of the settings
src/scorer.rs         ScoreSettings holds the route and the two origins;
                      ScoreError::failure
tests/gateway.rs      new: CRITERION-94, CRITERION-96, the sources, the key of
                      each route, the check
tests/common/         FakeServer::root; the helpers set JAG_GATEWAY_API_KEY
tests/failure.rs, tests/check.rs, tests/cost_limit.rs, tests/usage.rs,
tests/score.rs, tests/setup.rs, tests/secrecy.rs, tests/output.rs
                      see "Tests that change"
README.md             the section "Use an AI gateway"
```

The code of `src/scorer/client.rs` does not change. It calls
`<base_url>/systemone`, and the address rule gives a base URL that ends with
`/v1`.

### The address rule (C-config)

One function gives the base URL from the text of any source:

```rust
/// The address rule (decisions 7 and 8). `None` when jag refuses the address.
pub fn normalize_base_url(text: &str) -> Option<String>
```

The steps, in this order:

1. Remove the white space at the start and at the end.
2. Remove each `/` at the end.
3. Refuse the text when it holds `?` or `#`. jag examines the text, not the
   parsed address.
4. Refuse the text when `address_is_allowed` refuses it (FEATURE-04 spec
   decision 29, as now).
5. When the path of the address does not end with `/v1`, add `/v1`.

The path is the part after the host and the port. The compare is exact, so
`/V1` and `/v1beta` do not end with `/v1`.

| Text | Base URL | Call to |
|---|---|---|
| `https://api.typesafe.ai` | `https://api.typesafe.ai/v1` | `https://api.typesafe.ai/v1/systemone` |
| `https://api.typesafe.ai/` | `https://api.typesafe.ai/v1` | the same |
| `https://api.typesafe.ai/v1` (the default) | the same | the same |
| `https://openrouter.ai/api` | `https://openrouter.ai/api/v1` | `https://openrouter.ai/api/v1/systemone` |
| `https://ai-gateway.vercel.sh/typesafe` | `https://ai-gateway.vercel.sh/typesafe/v1` | `.../typesafe/v1/systemone` |
| `https://gateway-us.pydantic.dev/proxy/typesafe` | `https://gateway-us.pydantic.dev/proxy/typesafe/v1` | `.../proxy/typesafe/v1/systemone` |
| `http://127.0.0.1:8080/v1` (the fake server) | the same | `http://127.0.0.1:8080/v1/systemone` |
| `https://example.com/v1beta` | `https://example.com/v1beta/v1` | `.../v1beta/v1/systemone` |

jag refuses these texts: `https://example.com/v1?a=1`, `https://example.com/#a`,
`https://user@example.com`, `http://example.com`, `ftp://example.com`,
`example.com`, and a blank text.

### The route (C-config)

```rust
/// Where the calls go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Route {
    /// The TypeSafe API (IF-1).
    #[default]
    Direct,
    /// An AI gateway (IF-2).
    Gateway,
}
```

The route is `Direct` when the base URL, after the address rule, uses
`https`, has the host `api.typesafe.ai` in any case, has no port or port
443, and has the path `/v1`. Each other base URL gives `Gateway`. So the
fake server of the tests, on `127.0.0.1`, is a gateway.

### The sources of the two settings (C-config)

```rust
/// Where the value of the base URL or of the model came from (CV-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Origin {
    Option,
    Variable,
    ConfigFile(PathBuf),
    #[default]
    Default,
}

/// The options of step 1, after their checks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingOptions {
    /// After the address rule.
    pub base_url: Option<String>,
    /// Trimmed. Not blank.
    pub model: Option<String>,
    /// `(timeout, retry wait)`, as now.
    pub times: (Option<Duration>, Option<Duration>),
}

pub struct ModelSettings {
    /// After the address rule, so it ends with `/v1`.
    pub base_url: String,
    pub base_url_origin: Origin,
    pub model: String,
    pub model_origin: Origin,
    pub route: Route,
    pub times: Times,
}

pub fn model_settings(
    env: &Env,
    file: &ConfigFile,
    options: &SettingOptions,
) -> Result<ModelSettings, ConfigError>
```

`model_settings` finds the base URL, then the model, then the timeout, then
the retry wait. For the base URL and for the model, it takes the first
source that is set:

1. The option. Step 1 checked it.
2. The variable. A blank variable counts as not set. A variable that is not
   UTF-8, or a base URL that the address rule refuses, gives exit 2.
3. The value in the config file. `model_settings` calls `file.get()` only
   here, so jag reads the file only when a setting needs it. A file error
   gives its exit 3. A blank or empty value counts as not set. A base URL
   that the address rule refuses gives exit 2.
4. The default.

The times keep their order: the option, the variable, the default. They
have no key in the file (DF-1).

A source that wins hides each lower source of the same setting. So
`--base-url` with a good value and a bad `JAG_BASE_URL` gives no error.
jag does not read a value that it does not use.

The variables keep the text of today: trimmed, and with no `/` at the end.
The address rule now does this for each source.

### The config file (C-config)

The file form of FEATURE-01 (decision 8) gets three keys. Each key is
optional:

```yaml
config:
  api-key: <the TypeSafe key>
  gateway-api-key: <the key of the gateway>
  base-url: "https://openrouter.ai/api"
  model: "~typesafe/jev-latest"
```

`Section` and `StrictSection` get the three fields:

```rust
#[serde(rename = "gateway-api-key")]
pub(crate) gateway_api_key: Option<String>,
#[serde(rename = "base-url")]
pub(crate) base_url: Option<String>,
pub(crate) model: Option<String>,
```

A search still ignores other keys (FEATURE-01 decision 18). `--setup` and
`--forget-key` still refuse a file with another key.

One run reads the file at most one time (decision 12). A new type keeps the
result of the read:

```rust
/// The config file of one run. The settings step and the key step share
/// the read (decision 12).
pub struct ConfigFile<'a> {
    env: &'a Env<'a>,
    fs: &'a dyn Fs,
    read: std::cell::OnceCell<Result<Option<(PathBuf, File)>, ConfigError>>,
}

impl<'a> ConfigFile<'a> {
    pub fn new(env: &'a Env<'a>, fs: &'a dyn Fs) -> Self;

    /// The path and the parse of the file. `Ok(None)` when there is no
    /// home or no file. The first call opens the file, checks the mode of
    /// the open file, reads it, and parses it. Each later call gives the
    /// same result and reads nothing.
    pub(crate) fn get(&self) -> Result<Option<(&Path, &File)>, ConfigError>;

    pub(crate) fn fs(&self) -> &dyn Fs;
}
```

`get` runs the steps of the file part of `find_key` of today, in the same
order: `open_key_file`, the mode of the open file (`mode & 0o077`), the
read, and `parse_loose`. Each error has the FEATURE-01 text and exit 3.
Item 5 of the check opens the file for its mode only, as now. It reads no
text, so it is not a second read.

`find_key` gives the key of the route (see "The key of each route"). It
calls `file.get()` only when no key variable of the route is set.

The mode check comes before each read, also when jag reads the file for
the settings only (decision 13). A person who can write to the file can
set `config.base-url`, and then jag sends the gateway key to that address.
A person who can read the file can read a key in it.

A new method names the file errors:

```rust
impl ConfigError {
    /// True for KeyFileOpen, KeyFileUnreadable, KeyFileNotRegular, and
    /// KeyFileBad.
    pub fn is_file_error(&self) -> bool;
}
```

### The key of each route (C-config)

```rust
pub const GATEWAY_VARIABLE: &str = "JAG_GATEWAY_API_KEY";

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

/// The key of the route (decision 11).
pub fn find_key(
    env: &Env,
    file: &ConfigFile,
    route: Route,
    base_url_origin: &Origin,
) -> Result<(ApiKey, KeySource), ConfigError>
```

| Route | Sources, in order | No key |
|---|---|---|
| `Direct` | `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, then `config.api-key` (DECISION-03, as now) | `KeyMissing`, as now |
| `Gateway` | `JAG_GATEWAY_API_KEY`, then `config.gateway-api-key` (0017) | `GatewayKeyMissing { by_option }` |

- Each source keeps the rules of FEATURE-01. A blank variable, or a blank
  value in the file, counts as not set. A variable that is not UTF-8, or a
  key with a space or a character that is not plain ASCII, gives
  `KeyFormatBad { source }` with exit 3, and stops the lookup (FEATURE-01
  spec decisions 14, 15, and 16).
- `find_key` never reads a source of the other route. So a bad
  `JAG_TYPESAFE_API_KEY` does not stop a search on the gateway route, and a
  bad `JAG_GATEWAY_API_KEY` or `config.gateway-api-key` does not stop a
  search on the direct route.
- `by_option` is true when `base_url_origin` is `Origin::Option`. It
  selects the fix of the error (see "Failure texts").

`Env::set_variable()` becomes `Env::set_key_variable(route: Route) ->
Option<&'static str>`: the first key variable of the route that is set and
not blank. Item 1 of the check and the notes of `--setup` and
`--forget-key` use it.

### The command line (C-cli)

```rust
/// The address of the model service or of an AI gateway (default https://api.typesafe.ai/v1)
#[arg(long = "base-url", value_name = "URL", conflicts_with_all = ["setup", "forget_key"])]
base_url: Option<String>,

/// The model name to send (default jev-1.13.0)
#[arg(long, value_name = "NAME", conflicts_with_all = ["setup", "forget_key"])]
model: Option<String>,
```

The `--check` list of conflicts does not change, so `--check` takes the two
options.

The usage lines (decision 22). The search lines of FEATURE-12 stay, and the
check line gets the two options:

```
jag [OPTIONS] TERM [PATH...]
jag [OPTIONS] --ask QUESTION TERM [PATH...]
jag --check [--timeout SECONDS] [--retry-wait SECONDS] [--base-url URL] [--model NAME]
jag --setup
jag --forget-key
```

The FEATURE-13 spec adds the line `jag [OPTIONS] -e TERM ... [PATH...]`
after the second line. The code that merges later keeps each line.

The help of `--setup` and `--forget-key` names the key of the route:

```rust
/// Store your API key, with no echo: the TypeSafe key, or the gateway key when JAG_BASE_URL or config.base-url names a gateway
#[arg(long, conflicts_with_all = ["forget_key", "term", "ask", "paths"])]
setup: bool,

/// Remove the stored key: the TypeSafe key, or the gateway key when JAG_BASE_URL or config.base-url names a gateway
#[arg(long = "forget-key", conflicts_with_all = ["term", "ask", "paths"])]
forget_key: bool,
```

Step 1 of a search checks the options in this order: `--timeout`,
`--retry-wait`, `--max-scored`, `--max-chars` or `--no-max-chars`,
`--base-url`, and `--model`. `--check` checks its options in this
order: `--timeout`, `--retry-wait`, `--base-url`, and `--model`.

- `--base-url`: the address rule. A refused value gives
  `BaseUrlBad { source: SettingSource::Option }`, exit 2.
- `--model`: trimmed. A blank value gives
  `ModelBad { source: SettingSource::Option }`, exit 2.

`SearchArgs` holds a `SettingOptions` in place of `times`.

### The search flow (C-cli)

The steps of the FEATURE-04 spec stay, but the settings now come before
the key (decision 27):

```rust
let args = search_args(cli)?; // step 1
// The option is known now. The variable comes with the other settings.
if let Some(option) = args.size_limit {
    stop.set_size_limit(option.chars());
}
// With no question, jag works as grep: no key, no model setting, and no
// model call (FEATURE-12, decision 4).
let Some(question) = args.question else {
    return plain(cli, &args, env, context, summary, stdout, stderr);
};
// The settings and the key come before the scan (FEATURE-04 decision 18,
// ADR-0017).
let file = ConfigFile::new(env, &RealFs);
let settings = model_settings(env, &file, &args.settings).map_err(Failure::from)?; // step 2
let (key, _source) = find_key(env, &file, settings.route, &settings.base_url_origin)
    .map_err(Failure::from)?; // step 3
let limit = cost_limit(env, args.cost_limit).map_err(Failure::from)?;
let size = size_limit(env, args.size_limit)
    .map_err(Failure::from)?
    .chars();
stop.set_size_limit(size);
```

Step 2 can give exit 3: a file error, when a setting needs the file. Then
jag does not look for the key. When step 2 read the file, step 3 uses the
same read. The cost limit and the size limit come after the key, as now.
The comment of today, "The key comes before any file read (decision 18).",
becomes the comment above step 2.

The order of the failures of steps 1 to 3:

| Case | Exit |
|---|---|
| A bad option (step 1) | 2 |
| A bad setting variable or a bad `config.base-url`, with no key | 2 |
| A config file that others can read, or that is not valid YAML, when a setting or the key needs it | 3 |
| Good settings and no key for the route | 3 |

A bad `JAG_MAX_SCORED` or `JAG_MAX_CHARS` with no key still gives exit 3,
because the two limits come after the key.

A score failure becomes a `Failure` through `ScoreError::failure`:

```rust
let score_settings = ScoreSettings::new(settings);
let scores = score(&snippets, args.question, &key, &score_settings)
    .map_err(|error| error.failure(&score_settings))?;
```

### The readiness check (C-cli, C-check)

The check shows what a search does with the same settings (decision 20).
`cli::check` runs these steps:

1. Check `--timeout`, `--retry-wait`, `--base-url`, and `--model`. A bad
   value gives exit 2 and no item line.
2. Make the `ConfigFile`, and call `model_settings`.
   - `Ok`: the check finds the key of the route, and makes its call with
     these settings.
   - An error with `is_file_error()`: the check goes on. The items show the
     error (see the table below).
   - Each other error, a bad value of a variable or of the file: exit 2 and
     no item line (FEATURE-02 spec decision 17).
3. `check::run(env, &file, settings, &|key, settings| probe(key, settings))`.

The new signature:

```rust
pub fn run(
    env: &Env,
    file: &ConfigFile,
    settings: Result<ScoreSettings, ConfigError>,
    call: &dyn Fn(&ApiKey, &ScoreSettings) -> Result<(), ScoreError>,
) -> Report
```

Item 5 calls `key_file_mode(env, file.fs())`, as now. Items 1 to 4 follow
this table:

| The settings | Item 5 | Item 1 | Items 2, 3, and 4 |
|---|---|---|---|
| a file error | fail | not checked, `(jag cannot use the key file: see the last item)` | not checked, `(jag has no key to send)` |
| a file error | pass | fail, with the file error (exit 3) | not checked, `(jag has no key to send)` |
| `Ok` | fail, and `env.set_key_variable(route)` is `None` | not checked, `(jag cannot use the key file: see the last item)` | not checked, `(jag has no key to send)` |
| `Ok` | each other case | `find_key(env, file, route, &base_url_origin)`: pass with the source, or fail | from the call when item 1 passes, as now. Else not checked, `(jag has no key to send)` |

The two rows with `Ok` are the rule of today (FEATURE-02 spec decision 10),
with the key variable of the route. With a file error, jag does not know
the route, so it cannot look for the key. FEATURE-02 shows a problem of the
key file on item 1, so the check needs no new note.

The note of an item 1 that passes names the source: `(JAG_GATEWAY_API_KEY)`
or `(<path>)` on the gateway route, and the names of today on the direct
route.

The item texts stay. "the service" fits the TypeSafe API and a gateway.
The failure blocks use the texts of the route (see "Failure texts").

Example 1: `JAG_TYPESAFE_API_KEY` is set, no setting variable is set, and
`config.yml` has mode 0600 and holds YAML that is not valid. The check
makes no call and exits with 3:

```
fail        a key is present
not checked the service accepts the key (jag has no key to send)
not checked the account has credit (jag has no key to send)
not checked the service is reachable (jag has no key to send)
pass        other users cannot read the key file
```

The error stream:

```
jag: /home/user/.config/jag/config.yml is not valid YAML (line 1).
Fix the file by hand, or remove it and run `jag --setup`.
```

Example 2: the same, but the file is valid and has mode 0644. The check
makes no call and exits with 3:

```
not checked a key is present (jag cannot use the key file: see the last item)
not checked the service accepts the key (jag has no key to send)
not checked the account has credit (jag has no key to send)
not checked the service is reachable (jag has no key to send)
fail        other users cannot read the key file
```

The error stream:

```
jag: /home/user/.config/jag/config.yml can be read by other users (mode 0644).
Run `chmod 600 /home/user/.config/jag/config.yml`.
```

Before this spec, a check with a key variable did not read the file, so
the checks of examples 1 and 2 made their call. Risk 1 covers this change.

Example 3: `JAG_BASE_URL` names a gateway, `JAG_TYPESAFE_API_KEY` is set,
no gateway key is set, and there is no config file. The check makes no
call and exits with 3:

```
fail        a key is present
not checked the service accepts the key (jag has no key to send)
not checked the account has credit (jag has no key to send)
not checked the service is reachable (jag has no key to send)
pass        other users cannot read the key file (no key file)
```

The error stream:

```
jag: no gateway key found.
Get a key from the gateway, then run `jag --setup`,
or set JAG_GATEWAY_API_KEY.
```

When `--base-url` or `JAG_BASE_URL`, and `--model` or `JAG_MODEL`, are set,
no setting needs the file. Then the check reads the file only for the key,
as today.

### `--setup` and `--forget-key` (C-config)

`ChangeState` gets each value of the file, as the loose parse gives it, in
place of `has_key`:

```rust
/// The values of the file, to write them again. A blank or empty value
/// counts as not set. `Debug` shows only which values are set, because two
/// of them are keys (CV-6).
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FileValues {
    pub api_key: Option<String>,
    pub gateway_api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeState {
    pub exists: bool,
    pub mode: u32,
    pub values: FileValues,
}
```

jag keeps each value that is set as the file gives it, with no trim and no
check. jag does not write a blank or empty value again.

`write_key_file` writes the values:

```rust
pub fn write_key_file(
    fs: &dyn Fs,
    home: &Path,
    values: &FileValues,
) -> Result<PathBuf, ConfigError>
```

The text is `config:\n`, then one line for each value that is set, in
this order: `  api-key: <scalar>\n`, `  gateway-api-key: <scalar>\n`,
`  base-url: <scalar>\n`, and `  model: <scalar>\n`. The temporary file,
the mode 0600, and the rename stay as now.

`yaml_scalar` keeps its plain form and its quoted form. In the quoted form,
`\` and `"` get a `\` in front, as now. Each character from U+0000 to
U+001F, from U+007F to U+009F, and U+FEFF, U+FFFE, and U+FFFF goes as
`\uXXXX`. So each kept value reads back as the same text. A URL has a `:`,
so it goes in quotes. A model name that starts with `~` goes in quotes.

The two options work on the key of the route (decision 15):

```rust
/// The route from `JAG_BASE_URL`, then `config.base-url` of the file, then
/// the default. `--setup` and `--forget-key` take no `--base-url`.
fn change_route(env: &Env, path: &Path, values: &FileValues) -> Result<(Route, Origin), ConfigError>
```

The address rule and its errors apply, as for a search. A bad
`JAG_BASE_URL` gives `BaseUrlBad { source: SettingSource::Variable }`, and
a bad `config.base-url` gives `FileBaseUrlBad { path }`. Both give exit 2,
and the file does not change.

The texts of each route:

| Part | Direct route | Gateway route |
|---|---|---|
| The key of the file | `config.api-key` | `config.gateway-api-key` |
| The line before the question | none | `JAG_BASE_URL names an AI gateway, so jag stores a gateway key.`, or `config.base-url in <path> names an AI gateway, so jag stores a gateway key.` |
| The question when the file holds the key | `A key is already stored in <path>. Replace it? [y/N] ` (as now) | `A gateway key is already stored in <path>. Replace it? [y/N] ` |
| The answer no | `Kept the stored key.` (as now) | `Kept the stored gateway key.` |
| The prompt | `TypeSafe API key: ` (as now) | `Gateway API key: ` |
| A typed key with a bad format | `KeyTypedBad { route: Route::Direct }`, with the text of today | `KeyTypedBad { route: Route::Gateway }` (see "Failure texts") |
| After the write | `Stored the key in <path>.` (as now) | `Stored the gateway key in <path>.` |
| `--forget-key` with no key of the route | `No stored key in <path>.` (as now) | `No stored gateway key in <path>.` |
| `--forget-key` after the removal | `Removed the stored key from <path>.` (as now) | `Removed the stored gateway key from <path>.` |
| The note when a key variable of the route is set | `Note: <name> is set, and it wins over the stored key.` and `Note: <name> is still set, so jag still has a key.` (as now) | the same texts, with `JAG_GATEWAY_API_KEY` |

`--setup` runs these steps:

1. No terminal: `NoTerminal`, exit 2, as now.
2. `load_for_change`, as now.
3. A file that others can read: `KeyFileOpen`, exit 3, as now.
4. The route, from `change_route`. A bad value gives exit 2.
5. On the gateway route, the line before the question, on standard output.
6. When the file holds the key of the route, the question. The answer no
   writes its line, and the file does not change.
7. The prompt, with no echo.
8. The format check. A bad key gives `KeyTypedBad { route }`, exit 2.
9. `write_key_file`, with the new key in the field of the route and each
   other value as it was.
10. The message, and the note from `env.set_key_variable(route)`.

`--forget-key` runs these steps:

1. No home: `No stored key: HOME is not set.`, as now.
2. `load_for_change`, as now.
3. The route, from `change_route`. A bad value gives exit 2.
4. No file, or no key of the route in the file: the message, exit 0.
5. jag clears the field of the route. When no value is left, jag removes the
   file, as now. Else jag writes the file again with `write_key_file`.
6. The message, and the note from `env.set_key_variable(route)`.

To store or remove the key of the other route, the user sets
`JAG_BASE_URL` for one command. For example,
`JAG_BASE_URL=https://api.typesafe.ai jag --setup` stores the TypeSafe key
when `config.base-url` names a gateway. The README says so.

The text of `KeyFileHasOtherKeys` names the four keys:

| Cause | Fix |
|---|---|
| `<path> holds settings other than config.api-key, config.gateway-api-key, config.base-url, and config.model.` | `Remove the other settings, or edit the file by hand.` (as now) |

### Failure texts (C-config, C-scorer)

The bad values (exit 2). No text shows the value:

| Error | Cause | Fix |
|---|---|---|
| `BaseUrlBad { source: Option }` | `the value of --base-url is not an https address.` | `Use an https address, such as --base-url https://api.typesafe.ai.` |
| `BaseUrlBad { source: Variable }` | `JAG_BASE_URL is not an https address.` (as now) | `Set JAG_BASE_URL to an https address, or unset it.` (as now) |
| `FileBaseUrlBad { path }` | `config.base-url in <path> is not an https address.` | `Set config.base-url in <path> to an https address, or remove it.` |
| `ModelBad { source: Option }` | `the value of --model is blank.` | `Use a model name, such as --model jev-1.13.0.` |
| `ModelBad { source: Variable }` | `JAG_MODEL is not text.` (as now) | `Set JAG_MODEL to a model name, or unset it.` (as now) |

The two variants of today get a `source: SettingSource` field.
`FileBaseUrlBad` is new. A model name in the file is always text, so it has
no bad value.

The failures of a call. `impl From<ScoreError> for Failure` becomes:

```rust
impl ScoreError {
    /// The cause and the fix for the route and the sources of the settings.
    pub fn failure(self, settings: &ScoreSettings) -> Failure;
}
```

`ScoreSettings` gets three fields: `route: Route`, `base_url_origin:
Origin`, and `model_origin: Origin`. `ScoreSettings::new(settings:
ModelSettings)` copies them and computes the deadline, as now.

In the fixes, B is the name of the source of the base URL, and M is the
name of the source of the model:

| Origin | B | M |
|---|---|---|
| `Option` | `--base-url` | `--model` |
| `Variable` | `JAG_BASE_URL` | `JAG_MODEL` |
| `ConfigFile(path)` | `config.base-url in <path>` | `config.model in <path>` |
| `Default` | none | none |

The direct route. Each cause stays as now:

| Error | Cause | Fix |
|---|---|---|
| `KeyNotAccepted { status }` | `the TypeSafe API did not accept the key (HTTP <status>).` | `Check the key at https://console.typesafe.ai/keys, then run jag --setup.` |
| `NoCredit` | `the TypeSafe account has no credit (HTTP 402).` | `Add credit at https://console.typesafe.ai, then try again.` |
| `TimedOut` | `the TypeSafe API did not answer in time.` | `Try again later, or give each call more time with --timeout.` |
| `CannotConnect` | `jag cannot connect to the model service.` | B: `Check the network connection and <B>, then try again.` No B: `Check the network connection, then try again.` |
| `RateLimited` | `the TypeSafe API refused more calls (HTTP 429).` | `Wait one minute, then try again.` |
| `ServerFailed { status }` | `the TypeSafe API failed (HTTP <status>).` | `Try again later.` |
| `Redirect` | `the model service sent a redirect.` | B: `Check <B>.` No B: `Update jag, then try again.` |
| `UnexpectedStatus { status }` | `the TypeSafe API gave HTTP <status>.` | B and M: `Check <B> and <M>.` Only one: `Check <B>.` or `Check <M>.` None: `Update jag, then try again.` |
| `BadResponse` | `the TypeSafe API gave a response that jag cannot read.` | B: `Check <B>.` No B: `Update jag, then try again.` |

The gateway route. B is always set, because the default address is the
direct route:

| Error | Cause | Fix |
|---|---|---|
| `KeyNotAccepted { status }` | `the gateway did not accept the key (HTTP <status>).` | B is `--base-url`: `Check that the key is a key of the gateway.` Else: `Check that the key is a key of the gateway, then run jag --setup.` |
| `NoCredit` | `the gateway account has no credit (HTTP 402).` | `Add credit at the gateway, then try again.` |
| `TimedOut` | `the gateway did not answer in time.` | `Try again later, or give each call more time with --timeout.` |
| `CannotConnect` | `jag cannot connect to the gateway.` | `Check the network connection and <B>, then try again.` |
| `RateLimited` | `the gateway refused more calls (HTTP 429).` | `Wait one minute, then try again.` |
| `ServerFailed { status }` | `the gateway failed (HTTP <status>).` | `Try again later.` |
| `Redirect` | `the gateway sent a redirect.` | `Check <B>.` |
| `UnexpectedStatus { status }` | `the gateway gave HTTP <status>.` | M: `Check <B> and <M>.` No M: `Check <B>, and set the model name of the gateway with --model, JAG_MODEL, or config.model.` |
| `BadResponse` | `the gateway gave a response that jag cannot read.` | `Check <B>.` |

The exit codes stay: 4, 5, and 6.

The key texts (decision 18). The direct route keeps each text of today:
`KeyMissing`, `KeyFormatBad` with the three sources of DECISION-03, and
`KeyTypedBad { route: Route::Direct }`. The new texts of the gateway route:

| Error | Cause | Fix | Exit |
|---|---|---|---|
| `GatewayKeyMissing { by_option: false }` | `no gateway key found.` | two lines: ``Get a key from the gateway, then run `jag --setup`,`` and `or set JAG_GATEWAY_API_KEY.` | 3 |
| `GatewayKeyMissing { by_option: true }` | `no gateway key found.` | `Set JAG_GATEWAY_API_KEY to the key of the gateway.` | 3 |
| `KeyFormatBad { source: GatewayVariable }` | `the key in the variable JAG_GATEWAY_API_KEY has a space or a character that is not plain ASCII.` | `Set JAG_GATEWAY_API_KEY to the key of the gateway, or unset it.` | 3 |
| `KeyFormatBad { source: GatewayConfigFile(path) }` | `the gateway key in <path> has a space or a character that is not plain ASCII.` | `Fix config.gateway-api-key in <path>, or set JAG_GATEWAY_API_KEY.` | 3 |
| `KeyTypedBad { route: Route::Gateway }` | `the key has a space or a character that is not plain ASCII.` (as now) | `Copy the key again from the gateway.` | 2 |

With `--base-url`, `jag --setup` does not see the gateway (decision 15), so
the fix of `by_option: true` does not name `jag --setup`. The fix of
`KeyNotAccepted` on the gateway route follows the same rule.

Three texts of FEATURE-01 change, because a key variable no longer stops
each read of the file, and because a gateway has its own variable:

| Error | Cause | Fix | Exit |
|---|---|---|---|
| `KeyFileNotRegular { path }` | as now | ``Remove it, then run `jag --setup`.`` | 3 |
| `NoTerminal` | as now | two lines: `To give the key with no terminal, set JAG_TYPESAFE_API_KEY,` and `or JAG_GATEWAY_API_KEY for an AI gateway.` | 2 |
| `NoHome` | as now | two lines: `Set HOME, or set JAG_TYPESAFE_API_KEY,` and `or JAG_GATEWAY_API_KEY for an AI gateway.` | 2 |

The other file errors keep their texts: `KeyFileOpen`, `KeyFileUnreadable`,
`KeyFileBad`, `KeyFileWrite`, and `KeyFileIsLink`. They name the file and
no key variable.

### Secrecy (CV-6)

- No message shows the value of a base URL or of a model name, from any
  source. A message shows only the name of the source.
- The key goes only into the `Authorization` header, as now. No option
  takes the key.
- The address rule refuses a user name and a password, so the process list
  shows no secret when the base URL is an option.
- A key goes only on its own route (decision 11). A key from
  `JAG_TYPESAFE_API_KEY`, `TYPESAFE_API_KEY`, or `config.api-key` goes only
  to the TypeSafe API. Only a key from `JAG_GATEWAY_API_KEY` or
  `config.gateway-api-key` goes to a gateway. With no key for the route,
  jag stops before the call.
- `FileValues` holds the two keys of the file as text, so its `Debug`
  shows only which values are set. It has no `Display` and no `Serialize`.
- `--setup` reads the gateway key with no echo, as the TypeSafe key.

### README

A new section "Use an AI gateway", after "Where jag looks for the key":

- jag sends the same calls to `<base URL>/v1/systemone`, with the key of
  the gateway as a Bearer key.
- A table of the three gateways of the TypeSafe docs, with the base URL
  and the model name of each.

  | Gateway | Base URL | Model name |
  |---|---|---|
  | OpenRouter | `https://openrouter.ai/api` | `~typesafe/jev-latest` |
  | Vercel AI Gateway | `https://ai-gateway.vercel.sh/typesafe` | `typesafe-ai/jev` |
  | Pydantic AI Gateway | `https://gateway-us.pydantic.dev/proxy/typesafe` | `jev-latest` |

- The key of the gateway: jag reads `JAG_GATEWAY_API_KEY`, then
  `gateway-api-key` under `config:` in the file. `jag --setup` stores it
  when `JAG_BASE_URL` or `config.base-url` names the gateway. jag never
  sends the TypeSafe key to a gateway, and never sends the gateway key to
  TypeSafe.
- To store the key of the other route, set `JAG_BASE_URL` for one command,
  for example `JAG_BASE_URL=https://api.typesafe.ai jag --setup`.
- The three sources of the base URL and the model name: the options for one
  search, the variables for one job, and the config file for each search,
  with the file example of "The config file". The order of CRITERION-96.
- Set the model name too. The default model name is a TypeSafe name, and a
  gateway can use another name.
- Put a model name that starts with `~` in quotes, in the shell and in the
  file. A shell can change a word that starts with `~`.
- With no setting, jag calls TypeSafe directly.
- Each example has `--ask`, because with no question jag makes no call
  (FEATURE-12). For example:
  `jag --base-url https://openrouter.ai/api --model "~typesafe/jev-latest" --ask "how long is the wait" retry src`.

Lines of the README that change too:

- "Where jag looks for the key", after the list: "When the base URL names
  an AI gateway, jag looks only for the gateway key. See "Use an AI
  gateway"."
- The check section: "`--timeout`, `--retry-wait`, `--base-url`, and
  `--model` apply to the call."
- The exit code 3 row: "No key for the route, a bad key, or a key file that
  others can read or that jag cannot read".
- "Plain grep with no question": "With no question, `--timeout`,
  `--retry-wait`, `--max-scored`, `--base-url`, and `--model` have no
  effect. jag still checks their values." The code that merges later keeps
  the options of each merged feature in this line.

### Acceptance criteria and their tests

`FakeServer` gets `root()`: `http://127.0.0.1:<port>`, with no path. The
tests set `HOME` to a temporary folder, use the made-up key
`SENTINEL-SENTINEL-SENTINEL`, and call only fake servers.

- CRITERION-94 (jag sends the calls to the gateway that the user names):
  - End-to-end (tests/gateway.rs), one test for each source: the options
    (`--base-url` and `--model`), the variables (`JAG_BASE_URL` and
    `JAG_MODEL`), and the config file (`config.base-url` and `config.model`,
    mode 0600, the key in `JAG_GATEWAY_API_KEY`). The model name is
    `gw-model`. For each source and for each of these forms of the address:
    `root()`, `root()` with `/`, `root()` with `/v1`, `root()` with `/api`,
    and `root()` with `/proxy/typesafe/`. In the order of the forms, the
    fake server gets one `POST` to `/v1/systemone`, `/v1/systemone`,
    `/v1/systemone`, `/api/v1/systemone`, and
    `/proxy/typesafe/v1/systemone`. The body has `"model": "gw-model"`. The
    `authorization` header is `Bearer SENTINEL-SENTINEL-SENTINEL`. The
    search exits with 0.
  - End-to-end: `config.gateway-api-key` and the two settings in the config
    file, and no key variable. The call has the key of the file.
  - End-to-end: `jag --check --base-url <root()> --model gw-model`, with the
    key in `JAG_GATEWAY_API_KEY`. The fake server gets one call to
    `/v1/systemone` with `gw-model`. Exit 0.
  - Unit (src/config/model.rs): each row of the table of "The address
    rule", and each text that jag refuses, for each of the three sources.
  - Unit (src/config/model.rs): the route for `https://api.typesafe.ai/v1`,
    `https://API.TypeSafe.ai/v1`, and `https://api.typesafe.ai:443/v1`
    (direct), and for `https://api.typesafe.ai:8443/v1`,
    `https://openrouter.ai/api/v1`, and `http://127.0.0.1:8080/v1`
    (gateway).
- CRITERION-95 (a TypeSafe key alone is enough):
  - Unit (src/config/model.rs): with no option, no variable, and no file,
    `model_settings` gives `https://api.typesafe.ai/v1`, `jev-1.13.0`,
    `Origin::Default` for both, and `Route::Direct`. It needs no value
    other than the key.
  - Unit (src/config.rs): on the direct route, `find_key` with only
    `JAG_TYPESAFE_API_KEY`, only `TYPESAFE_API_KEY`, or only
    `config.api-key` gives that key. No gateway key is necessary.
  - Unit (src/scorer/client.rs): `Client::new` with these settings calls
    `https://api.typesafe.ai/v1/systemone`.
  - Unit (src/scorer.rs): with these settings, each failure has the direct
    text of the table, with "TypeSafe" in it, and no fix names a source.
- CRITERION-96 (an option wins over a variable, and a variable wins over
  the config file):
  - End-to-end (tests/gateway.rs), for the base URL, with two fake servers
    A and B: `--base-url A` and `JAG_BASE_URL=B` call A. `JAG_BASE_URL=A`
    and `config.base-url: B` call A. `config.base-url: A` and no option and
    no variable call A. B gets no call.
  - End-to-end, for the model: `--model m-option` and
    `JAG_MODEL=m-variable` send `m-option`. `JAG_MODEL=m-variable` and
    `config.model: m-file` send `m-variable`. `config.model: m-file` sends
    `m-file`. No source sends `jev-1.13.0`.
  - End-to-end: each setting has its own order. `--base-url A` and
    `config.model: m-file` call A with `m-file`.
  - End-to-end: a source that wins hides a bad lower value.
    `--base-url A` with `JAG_BASE_URL=http://example.com`, and
    `JAG_BASE_URL=A` with `config.base-url: "ftp://example.com"`, exit 0.
  - Unit (src/config/model.rs): each pair of sources for each setting,
    with `Origin` and the value.

The key of each route (decision 11):

- Unit (src/config.rs), on the gateway route: `JAG_GATEWAY_API_KEY` wins
  over `config.gateway-api-key`. `config.gateway-api-key` alone gives the
  key and `KeySource::GatewayConfigFile`. `JAG_TYPESAFE_API_KEY`,
  `TYPESAFE_API_KEY`, and `config.api-key`, together and with no gateway
  key, give `GatewayKeyMissing`. A blank `JAG_GATEWAY_API_KEY` counts as
  not set. `by_option` is true only for `Origin::Option`.
- Unit (src/config.rs), the other route is never read: on the gateway
  route, a `JAG_TYPESAFE_API_KEY` with a space and a good
  `JAG_GATEWAY_API_KEY` give the gateway key. On the direct route, a
  `JAG_GATEWAY_API_KEY` with a space and a good `JAG_TYPESAFE_API_KEY`
  give the TypeSafe key, and so does a bad `config.gateway-api-key`.
- Unit (src/config.rs): each row of the two key text tables of "Failure
  texts", with the exit code, and with no key text in the output.
- Unit (src/config.rs): `env.set_key_variable(route)` for each route, with
  the variables of both routes set.
- End-to-end (tests/gateway.rs): `JAG_BASE_URL=<root()>`,
  `JAG_TYPESAFE_API_KEY` and `TYPESAFE_API_KEY` set, `config.api-key` in
  the file, and no gateway key. Exit 3, the text of
  `GatewayKeyMissing { by_option: false }`, and the fake server gets no
  request. With `--base-url <root()>` in place of the variable: the text
  of `by_option: true`.
- End-to-end (tests/gateway.rs): `JAG_GATEWAY_API_KEY` set and a different
  made-up key in `config.gateway-api-key`. The call has the key of the
  variable.
- End-to-end (tests/usage.rs): the order of decision 27. A bad
  `JAG_TIMEOUT`, `JAG_RETRY_WAIT`, or `JAG_BASE_URL` with no key gives exit
  2. Good settings and no key give exit 3. A bad `JAG_MAX_SCORED` with no
  key gives exit 3, as now.

Other tests of this spec:

- Unit (src/config.rs): a fake `Fs` counts the opens and the reads. A run
  that needs the key and both settings from the file reads the file one
  time. A run with a key variable of the route and both setting variables
  does not open the file. A run with a key variable of the route and no
  setting variable reads it one time.
- Unit (src/config.rs): a file with mode 0644 and only `config.base-url`
  gives `KeyFileOpen` for a run with a key variable of the route and no
  `JAG_BASE_URL`. The text of the file is not read.
- Unit (src/config/change.rs): a file with the two keys and the two
  settings, and a file with only the two settings, read back the same after
  `write_key_file`. The values are `https://openrouter.ai/api`,
  `~typesafe/jev-latest`, `typesafe-ai/jev`, `jev-latest`, `null`,
  `a "b" \c`, a value with a tab, a line feed, and U+FEFF, and a value with
  `é`. The `Debug` text of `FileValues` holds no value.
- Unit (src/config/setup.rs), the direct route: `--setup` with the fake
  prompter replaces the TypeSafe key and keeps `config.gateway-api-key`,
  `config.base-url`, and `config.model`. `--forget-key` keeps them,
  removes the TypeSafe key, and the new file has mode 0600. `--forget-key`
  with only a key removes the file, as now.
- Unit (src/config/setup.rs), the gateway route: with `JAG_BASE_URL` on a
  gateway, `--setup` writes the line before the question, prompts
  `Gateway API key: `, stores `config.gateway-api-key`, keeps the other
  values, and writes `Stored the gateway key in <path>.`. With
  `config.base-url` on a gateway and no variable, the line names
  `config.base-url in <path>`. With a gateway key in the file, the
  question names the gateway key, and the answer no keeps the file.
  `--forget-key` removes only `config.gateway-api-key`. The notes name
  `JAG_GATEWAY_API_KEY`. A bad `JAG_BASE_URL` or a bad `config.base-url`
  gives exit 2, and the file does not change.
- End-to-end (tests/setup.rs): `jag --forget-key` on a file with the
  TypeSafe key and the two settings, where `config.base-url` is
  `https://api.typesafe.ai`. Exit 0. The file stays with the two settings
  and no key.
- End-to-end (tests/setup.rs): `jag --forget-key` with
  `JAG_BASE_URL=https://openrouter.ai/api`, on a file with both keys. Exit
  0, `Removed the stored gateway key from <path>.`, and the file keeps
  `config.api-key`. No call goes out.
- Unit (src/check.rs): the four rows of the table of "The readiness check",
  with the count of calls, and the note of item 1 for each source of the
  gateway key.
- End-to-end (tests/gateway.rs): the three examples of "The readiness
  check", with the exact lines, the exact block, exit 3, and no call. In
  examples 1 and 2, `JAG_BASE_URL` names the fake server and the key is in
  `JAG_GATEWAY_API_KEY`, so that no call can go to the real service. The
  lines and the blocks are the same.
- End-to-end (tests/usage.rs): `--setup --base-url X` and
  `--forget-key --model X` are argument errors (exit 2). A blank
  `--base-url` and a blank `--model` give exit 2 with the texts of the
  table.
- End-to-end (tests/secrecy.rs): a base URL with
  `SENTINEL-SENTINEL-SENTINEL` in its user name, as an option, as a
  variable, and in the file, gives exit 2, and no output holds `SENTINEL`.
  A 422 from the fake server with `--model SENTINEL-SENTINEL-SENTINEL`
  gives exit 6, and no output holds `SENTINEL`. A gateway key with a space
  in `JAG_GATEWAY_API_KEY` and in `config.gateway-api-key` gives exit 3,
  and no output holds `SENTINEL`.
- Unit (src/scorer.rs): each cell of the two failure tables, with each kind
  of origin.

### Tests that change

The fake server is on `127.0.0.1`, so the end-to-end tests are on the
gateway route. They need the gateway key, and they get the gateway texts.
`JAG_BASE_URL` names the fake server, and `JAG_MODEL` is not set:

- tests/common/mod.rs: `jag` and `jag_std` set `JAG_GATEWAY_API_KEY` to
  `KEY` in place of `JAG_TYPESAFE_API_KEY`. `jag_std` also removes
  `JAG_TYPESAFE_API_KEY`, and `jag_bare` also removes
  `JAG_GATEWAY_API_KEY`. So each end-to-end test that calls the fake server
  sends the gateway key, and each check of the `authorization` header stays.
- tests/output.rs
  `criterion_51_a_document_that_cannot_be_written_gives_the_text_failure_and_exit_2`:
  its own command sets `JAG_GATEWAY_API_KEY` in place of
  `JAG_TYPESAFE_API_KEY`.
- tests/failure.rs:
  - `http_401_and_403_give_exit_4`: `the gateway did not accept the key
    (HTTP <status>).` and `Check that the key is a key of the gateway,
    then run jag --setup.`
  - `http_402_gives_exit_5`, and the 402 test near the end of the file:
    `the gateway account has no credit (HTTP 402).` and `Add credit at the
    gateway, then try again.`
  - `a_timeout_gives_exit_6_after_one_retry`: `the gateway did not answer
    in time.`
  - `no_connection_gives_exit_6`: `jag cannot connect to the gateway.` The
    fix stays.
  - `http_429_gives_exit_6_after_one_retry`: `the gateway refused more
    calls (HTTP 429).`
  - `http_5xx_gives_exit_6_after_one_retry`: `the gateway failed (HTTP
    503).`
  - `a_redirect_gives_exit_6`: `the gateway sent a redirect.` and
    `Check JAG_BASE_URL.`
  - `another_status_gives_exit_6`: `the gateway gave HTTP 422.` and
    `Check JAG_BASE_URL, and set the model name of the gateway with
    --model, JAG_MODEL, or config.model.`
  - `a_body_that_jag_cannot_read_gives_exit_6`: `the gateway gave a
    response that jag cannot read.` and `Check JAG_BASE_URL.`
  - `a_500_during_the_search_fails_the_full_search`: `the gateway failed
    (HTTP 500).`
  - `no_key_gives_exit_3_and_the_setup_fix` does not change: it sets no
    `JAG_BASE_URL`, so it is on the direct route.
- tests/cost_limit.rs `a_model_failure_with_the_limit_gives_exit_6_and_the_notice`:
  `the gateway failed (HTTP 500).`
- tests/check.rs:
  - `check_with_key` also sets `JAG_MODEL` to `jev-1.13.0`. Then no
    setting needs the file, and the tests with an open key file and a key
    variable keep their items.
  - `READY`, and each line `pass        a key is present
    (JAG_TYPESAFE_API_KEY)`: `(JAG_GATEWAY_API_KEY)`.
  - `KEY_FILE_TEXT`: `config:\n  gateway-api-key: SENTINEL-SENTINEL-SENTINEL\n`.
    So `criterion_09_a_private_key_file_is_named_as_the_source` and
    `criterion_10_an_open_key_file_that_is_the_source_shows_its_block_one_time`
    keep their lines.
  - `criterion_10_no_key_names_the_problem_and_the_fix_and_makes_no_call`:
    the block of example 3 of "The readiness check", the text of
    `GatewayKeyMissing { by_option: false }`.
  - `NO_CREDIT_BLOCK`: `jag: the gateway account has no credit (HTTP
    402).\nAdd credit at the gateway, then try again.\n`.
  - `criterion_10_a_refused_key_fails_item_2_with_exit_4`: `jag: the
    gateway did not accept the key (HTTP 401).\nCheck that the key is a key
    of the gateway, then run jag --setup.\n`.
  - `criterion_10_a_service_that_fails_fails_item_4_with_exit_6`: `jag: the
    gateway failed (HTTP 500).\nTry again later.\n`.
  - `criterion_67_a_refused_key_and_an_open_key_file_exit_with_4`: the
    block starts with `jag: the gateway did not accept`.
  - `criterion_67_a_call_that_times_out_fails_item_4_with_exit_6`: `jag:
    the gateway did not answer in time.\n` and the fix as now.
- tests/usage.rs:
  - `help_and_version_exit_0`: the check line with `[--base-url URL]
    [--model NAME]`, and `--base-url <URL>` and `--model <NAME>` in the
    help.
  - `a_bad_time_option_comes_before_the_key_and_a_bad_variable_comes_after`:
    a bad `JAG_TIMEOUT` or `JAG_RETRY_WAIT` with no key now gives exit 2
    (decision 27). It gets the new name
    `a_bad_time_option_or_variable_comes_before_the_key`.
  - `a_bad_limit_option_comes_before_the_key_and_a_bad_variable_after` and
    `a_bad_size_option_comes_before_the_key_and_a_bad_variable_after` do not
    change: the cost limit and the size limit come after the key.
- tests/size_limit.rs does not change. Its tests with the fake server use
  the helpers of tests/common. `an_error_document_with_a_long_key_file_path_fits_and_parses`
  (a key file with mode 0644, and no variable) keeps exit 3 and the same
  `KeyFileOpen` text: step 2 now reads the file and gives the error.
  `plain_grep_reads_no_key_so_a_bad_variable_gives_exit_2` keeps exit 3
  with `--ask`, because the size limit comes after the key.
- tests/setup.rs and tests/secrecy.rs: their `jag` helpers also remove
  `JAG_GATEWAY_API_KEY` and `JAG_BASE_URL`, so that a variable of the test
  machine does not change the route. tests/setup.rs calls
  `find_key(&env, &ConfigFile::new(&env, &RealFs), Route::Direct,
  &Origin::Default)`.
- src/check.rs. The test helper makes a `ConfigFile` and calls
  `model_settings`, as `cli::check` does. `GOOD` sets no setting variable,
  so these tests are on the direct route:
  - `a_key_in_a_variable_and_a_private_key_file_pass` (YAML that is not
    valid, mode 0600, and a key variable): `[Fail, NotChecked, NotChecked,
    NotChecked, Pass]`, exit 3, and no call. It gets the new name
    `a_key_in_a_variable_and_a_key_file_that_is_not_yaml_fail_item_1`. A
    copy with `JAG_BASE_URL=https://api.typesafe.ai/v1` and
    `JAG_MODEL=jev-1.13.0` keeps the old name, `[Pass; 5]`, and one call.
  - `a_key_in_a_variable_and_an_open_key_file_fail_item_5_only` (mode 0644
    and a key variable): `[NotChecked, NotChecked, NotChecked, NotChecked,
    Fail]`, exit 3, and no call. A copy with the two setting variables
    keeps `[Pass, Pass, Pass, Pass, Fail]` and one call.
  - `a_bad_key_in_a_variable_and_an_open_key_file_fail_items_1_and_5` and
    `the_first_item_that_fails_gives_the_exit_code`: their variables get the
    two setting variables, so they keep their items and their codes.
- src/config/setup.rs: `setup_refuses_a_typed_key_with_a_bad_format` wants
  `KeyTypedBad { route: Route::Direct }`. The other tests stay on the
  direct route.
- src/scorer.rs: `each_failure_has_its_exit_code_and_no_value` uses
  `error.failure(&settings)` for both routes. The deadline tests build a
  `ModelSettings`.
- tests/score.rs: each `ScoreSettings` literal gets the three new fields.
- src/config.rs and src/config/model.rs: the helpers make a `ConfigFile`
  for `find_key` and `model_settings`, and give the route. The errors get
  their `source`.
- src/config/change.rs and src/config/setup.rs: the calls of
  `write_key_file` get a `FileValues`. The tests of `ChangeState` compare
  `values` in place of `has_key`.

After this change, no end-to-end test shows a text of the direct route.
The unit tests of the failure tables in `src/scorer.rs` cover each text of
the direct route. This is an accepted cost: only a hidden test setting could
put the fake server on the direct route, and that setting would add a route
that no user has.

## Build order

1. STORY-57, the route, the key of the route, and the texts (C-config,
   C-scorer, C-cli, C-check):
   1. `normalize_base_url` and `Route`, with their unit tests.
      `model_settings` uses the rule for `JAG_BASE_URL`.
   2. `Origin` in `ModelSettings` (`Variable` or `Default` for now). The
      order of decision 27: `model_settings`, then `find_key`. `find_key`
      for each route, with `KeySource`, `GatewayKeyMissing`,
      `config.gateway-api-key` in `Section` and `StrictSection`,
      `Env::set_key_variable`, and the key texts, with their unit tests.
      `find_key` reads the file as now, and `cli::check` and `check::run`
      give it the route. The helpers of tests/common set
      `JAG_GATEWAY_API_KEY`. The tests of "Tests that change" that set a key
      or pin the order of the steps.
   3. `ScoreSettings::new(ModelSettings)` and `ScoreError::failure`. The
      unit tests of the failure tables. The tests of "Tests that change"
      that pin a failure text.
   4. The CRITERION-94 tests with the variables, the CRITERION-95 tests, and
      the end-to-end tests of the key of each route.
2. STORY-58, the options and the config file (C-cli, C-config, C-check):
   1. `ConfigFile`, and `find_key(env, &file, route, &origin)`, with the
      tests of one read.
   2. `config.base-url` and `config.model` in the file, `FileBaseUrlBad`,
      and the order of `model_settings`, with the unit tests of the order.
   3. `--base-url` and `--model`: step 1, the usage lines, and the help.
   4. The readiness check: `run` with the result of the settings, the item
      table, and the three examples.
   5. `--setup` and `--forget-key`: `FileValues`, `write_key_file`, the
      escapes of `yaml_scalar`, `change_route`, the texts of each route, the
      help of the two options, and the text of `KeyFileHasOtherKeys`.
   6. The CRITERION-96 tests, the CRITERION-94 tests with the options and
      the file, and the secrecy tests.
   7. The README.
3. The merge with other work:
   - FEATURE-09 is on `main`: `--max-chars` and `--no-max-chars` come in
     step 1 before `--base-url`, and `JAG_MAX_CHARS` comes after the key.
     Its tests with the fake server use the helpers of tests/common, so
     they send the gateway key.
   - FEATURE-12 is on `main`. A search with no `--ask` reads no model
     setting and no key (decision 27), and it checks the two options in
     step 1 (decision 19).
   - FEATURE-14 (spec on `main`): its decision 9 and its flow list name
     the key before the settings. The code that merges second follows
     decision 27 and ADR 0017: the settings, then the key, then
     `build_matcher`, then the scan or the read of standard input.
   - FEATURE-16 (code on `main`): its measure, `scripts/speed/measure.py`,
     removes `JAG_BASE_URL` and `JAG_MODEL`, so that jag uses the direct
     route. After this spec, a `config.base-url` in the config file of the
     user can still give the gateway route. So the code that merges second
     makes the script always set `JAG_BASE_URL=https://api.typesafe.ai/v1`
     in the environment of jag, in place of the removal of `JAG_BASE_URL`.
     The script still never reads the config file or the key (CV-6). A user
     with only a gateway key gets exit 3 from jag, and the script stops with
     exit 2, because the measure needs a TypeSafe key. Check 1
     of its stand-ins then shows `JAG_BASE_URL` with this value, and no
     `JAG_MAX_CHARS`. A `config.model` in the file still applies on the
     direct route. When it is not a TypeSafe model name, jag fails, and the
     script stops with exit 2 and shows the error of jag.
   - FEATURE-13 (spec on `main`): its usage line of `-e` stays
     (decision 22).
   - FEATURE-20 (spec not merged): the cost of a call through a gateway. Its
     code keeps the order of decision 27.
   - FEATURE-11 is on `main`. Its tests in `tests/output.rs` compare the
     error document with the text of the text mode, and pin no failure
     text. Only the command of "Tests that change" gets the gateway key.
   - The code that merges second changes each test that sets
     `JAG_TYPESAFE_API_KEY` for the fake server to `JAG_GATEWAY_API_KEY`.

`cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` must
pass on the CI runs of `ubuntu-latest` and `macos-latest`.

## Assumptions

- The option names `--base-url` and `--model` (decision 2). From the pairs
  of options and variables in src/cli.rs and src/config/limit.rs.
- The README section (decision 25). From the FEATURE-11 spec, decision 29.
- No `?` and no `#` in a base URL (decision 8). jag adds `/systemone` at
  the end, so a query or a fragment would come before the path.
- One read of the config file in each run (decision 12). From src/cli.rs:
  the settings step and the key step read the same file.
- The route (decision 16). The TypeSafe docs give `https://api.typesafe.ai`
  as the address of the TypeSafe API, and each gateway has another host.
- `--check` item 1 fails for a file error of the settings read (decision
  20). With no route, jag cannot find the key, and src/check.rs shows a
  file error of the key read on item 1.
- `--setup` and `--forget-key` take the route from `JAG_BASE_URL` and
  `config.base-url` (decision 15). A user who names a gateway in a variable
  or in the file uses that gateway for each search, so that user stores the
  key of the gateway.
- The fix of `GatewayKeyMissing` with `--base-url` does not name
  `jag --setup` (decision 18), because `jag --setup` does not see the
  option (decision 15).
- The base URLs, the model names, and the keys of the three gateways come
  from the TypeSafe docs (`/typesafe-docs/typesafeai`, read on
  2026-10-13): each gateway takes its own key as the API key of the SDK,
  and the SDK sends it as a Bearer key to `<base URL>/v1/systemone`.
- `serde_saphyr` reads the `\uXXXX` escape of a double-quoted scalar, as
  YAML 1.2 says. The unit test of the kept values shows it.

## Risks

- A user with a key variable and a config file that others can read, or
  that is not valid YAML, goes from a working search to exit 3. Before,
  jag did not read the file when a key variable was set. Signal: an issue
  that shows exit 3 and the fix `chmod 600` after an update, from a user
  with a key variable.
- A user who moves from the direct route to a gateway has only the
  TypeSafe key, so the search stops with exit 3. The text names
  `JAG_GATEWAY_API_KEY` (0017). Signal: an issue about
  `no gateway key found.` from a user who set `JAG_TYPESAFE_API_KEY`.
- A shell sets `JAG_BASE_URL` by mistake, so `jag --setup` stores a gateway
  key in place of the TypeSafe key. The line before the prompt names the
  source of the gateway, and the prompt is `Gateway API key: `. Signal: a
  user reports `no TypeSafe API key found.` after `jag --setup`.
- A gateway needs a path that this rule does not give: two times `/v1`, or
  no `/v1/systemone`. Signal: `the gateway gave HTTP 404.` with a base URL
  from the docs of the gateway.
- The user sets only the base URL, so jag sends the pinned model name
  `jev-1.13.0` to a gateway that uses another name. Signal: `the gateway
  gave HTTP 400.` or `404` with the fix that tells the user to set the
  model name.
- A gateway gives no `usage` in its response. FEATURE-20 reads it
  (DECISION-45). Signal: no cost, or a cost of 0, for a search through a
  gateway.
- A gateway gives 401 or 403 for a wrong model name or a wrong path, so
  jag says that the key is wrong. Signal: a user reports the key text with
  a key that works in the console of the gateway.
- `--setup` and `--forget-key` do not keep the comments of the file, as
  now. Signal: a user reports a lost comment.

## Out of scope

- The other settings in the config file: the times, the cost limit, and
  the size limit. DF-1 stays for them. A later feature adds one when a
  user asks for it.
- On the direct route, the `--setup` prompt `TypeSafe API key: ` and the
  key texts stay.
- An option of `--setup` and `--forget-key` that selects the route, such as
  `--gateway`. `JAG_BASE_URL` for one command does the same (decision 15).
- jag-bench (C-bench): its feature uses the same settings.
- The cost of a call through a gateway: FEATURE-20.
- The system design: PR #144 (the architect) adds the gateway key, the two
  settings in the file, and the address rule (the ApiKey and Settings rows,
  CV-6, CV-7, IF-2, and DF-1).
- The PRD page DECISION-03: it gives the key order of the direct route. ADR
  0017 records the gateway route. The orchestrator keeps a PRD line for the
  gateway key as a later item for the user.
- Extra headers for a gateway. The TypeSafe docs need none.
