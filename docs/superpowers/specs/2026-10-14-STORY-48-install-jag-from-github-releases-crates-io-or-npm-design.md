---
title: "Install jag from GitHub Releases, crates.io, or npm: one release workflow for the three channels"
feature: FEATURE-18
stories: [STORY-48, STORY-49, STORY-50, STORY-51]
kind: build
gh_issue: 110
lifecycle_state: draft
inherits: [C-cli, IF-3, IF-4, IF-5, CV-1, CV-4, CV-6]
session: 84086903-189a-4495-bbe8-0cfb867635cd
---

# Install jag from GitHub Releases, crates.io, or npm: one release workflow for the three channels

## Problem

One version tag builds the jag binary for four platforms. The release then
publishes jag on GitHub Releases, crates.io, and npm with the same version
(FEATURE-18, `docs/adr/0010-ship-jag-on-github-releases-crates-io-and-npm.md`).

- STORY-48: "As a shell power user (PERSONA-04), I want a prebuilt binary for
  my platform on GitHub Releases, so that I can install jag in seconds with
  no runtime."
  - CRITERION-77: "Each release on GitHub Releases has one archive for Linux
    x86_64, Linux arm64, macOS arm64, and macOS x86_64, and each archive has
    a checksum."
  - CRITERION-78: "The binary from the archive runs on its platform with no
    other runtime or program installed."
- STORY-49: "As a shell power user (PERSONA-04), I want to install jag with
  cargo install, so that I can build jag from source with the Rust tools I
  use."
  - CRITERION-79: "cargo install jag-grep installs the jag command, with the
    same version as the GitHub Release."
- STORY-50: "As an agent builder (PERSONA-01), I want to install jag with npm
  or run it with npx, so that my agent setup gets jag with the tools it
  already uses."
  - CRITERION-80: "npx @airf/jag runs jag on each of the four platforms, and
    the install downloads nothing from GitHub."
  - CRITERION-81: "npm installs only the binary package that matches the
    platform of the machine."
- STORY-51: "As an agent builder (PERSONA-01), I want one version tag to
  publish jag on each channel with the same version and no private data, so
  that each channel gives the same jag."
  - CRITERION-82: "One version tag publishes the GitHub Release, the crate,
    and the npm packages, and each one has the same version."
  - CRITERION-83: "The release publishes to crates.io and npm with trusted
    publishing, and the repository stores no publish token."
  - CRITERION-84: "The archives, the crate, and the npm packages hold no key,
    no case content, no private evaluation data, and no internal project
    name."

METRIC-04: "A new user goes from install to a first good result in under
five minutes." SCOPE-01 is the first public release. CONSTRAINT-01: "The
public repository, its history, and the published package must contain no
private evaluation data, no case content, no keys, and no internal project
names."

The facts on 2026-10-14:

- The repository has no release workflow and no version tag. `main` has no
  `LICENSE` file. Pull request #181 adds it.
- `Cargo.toml` has no `include` list. So `cargo package` packs 439 files,
  with `docs/` and `CLAUDE.md` (relay:scout, in Docker `rust:1.99`).
- A test build in Docker `rust:1.99` for `x86_64-unknown-linux-musl`, with
  `musl-tools`, passed. `readelf` showed no `INTERP` header and no `NEEDED`
  entry. With an empty environment, the binary printed `jag 0.1.0`. A bad
  option gave exit 2. A term with no match in an empty folder gave exit 1.
- In the same test, the default rules of gitleaks 8.30.1 found nothing in
  the 88,177 strings of the binary (`strings -a -n 4`).
- In the same test, `cargo package --list` with the `include` list of
  decision 11 gave 26 paths. These are the files of `src/` and 5 files at
  the root. `cargo package` wrote `target/package/jag-grep-0.1.0.crate`, and
  `cargo publish --dry-run` passed.
- The crate name `jag-grep` is free on crates.io. `@airf/jag` does not exist
  on npm. The npm organization `airf` does not exist yet, and nothing has
  trusted publishing. CHORE-02 is open with the chores role.
- crates.io: "Your crate must already be published to crates.io (initial
  publish requires an API token)".
- npm (`npm trust`): "The package you're configuring must already exist on
  the npm registry." npm: "A newly created trusted publisher configuration
  must complete its first successful publish within 2 days."
- FEATURE-19 makes `ai-ready-future/jag` a public repository. FEATURE-18 is
  blocked by FEATURE-19, so the release runs only in the public repository.
  There, the standard runners of GitHub are free.

## Decisions

| # | Decision | Choice | Source |
|---|---|---|---|
| 1 | Channels | GitHub Releases: one archive with a checksum for each platform. crates.io: the crate `jag-grep`. npm: the package `@airf/jag` with a small launcher, and one package for each platform as an optional dependency. One tag publishes the three channels | `docs/adr/0010-ship-jag-on-github-releases-crates-io-and-npm.md` (accepted). System design deployment-1 |
| 2 | Platforms and Rust targets | Linux x86_64: `x86_64-unknown-linux-musl`. Linux arm64: `aarch64-unknown-linux-musl`. macOS arm64: `aarch64-apple-darwin`. macOS x86_64: `x86_64-apple-darwin`. The Linux binaries are static, because a musl target links the C library into the binary | CRITERION-77. System design deployment-2 (human:404pilo, 2026-10-12): "Linux x86_64 and arm64 (static)". `docs/adr/0005-build-jag-as-one-rust-binary.md`. musl is inferred. The test build of "Problem" |
| 3 | Build runners | One native runner for each target: `ubuntu-24.04`, `ubuntu-24.04-arm`, `macos-15`, and `macos-15-intel`. The labels are fixed, not `-latest`. Each job runs the tests and the binary on its own platform, with no cross compiler | Inferred from the runner table of GitHub (`supported-github-runners.md`, 2026-10-14). The runner images have Rust, Python 3, Node, and `readelf`. A CI annotation of 2026-10-14: `ubuntu-latest` moves to Ubuntu 26 from 2026-10-19 |
| 4 | Start of a release | A push of a version tag `v*` starts `.github/workflows/release.yml` (`on: push: tags`). The workflow builds the four binaries and attaches them to the GitHub Release. It publishes the crate and the npm packages with trusted publishing. The repository stores no publish token | System design deployment-3 (human:404pilo, 2026-10-12). CRITERION-82. CRITERION-83. FEATURE-19 spec decision 22: the event policy allows `push`, `pull_request`, and `workflow_dispatch` |
| 5 | Dry run | `workflow_dispatch` starts a dry run. A pull request that changes a release file also starts one. The release files: `release.yml`, `scripts/release/`, `npm/`, `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`. A dry run runs the jobs up to the scan, with `cargo publish --dry-run`. It publishes nothing and needs no environment | Inferred: a test of the full path before the merge and before the first tag, with no change on a registry. GitHub runs `workflow_dispatch` only for a workflow that is on the default branch |
| 6 | Version source | `version` in `Cargo.toml` is the one source. The tag must be `v` and that version, else the release stops before the build. The release writes the version into the five `package.json` files. Only the form `MAJOR.MINOR.PATCH` is valid, so a pre-release version stops the release | Inferred from CRITERION-82 and ADR-0010: "All channels must carry the same version". IF-3: semantic versions |
| 7 | The tag commit | The release stops when the commit of the tag is not on `main` | Inferred from deployment-3 (each pull request runs the tests and the private-data scan) and CONSTRAINT-01 |
| 8 | Who pushes a tag, and tag rules | Only the user pushes a `v*` tag, or a session on a TASK of the orchestrator that names the version. A tag ruleset stops the update and the deletion of each `v*` tag, with no bypass. The repository turns on immutable releases | Inferred: a publish cannot be undone. GitHub docs: "Immutable releases" |
| 9 | Environment | The jobs that publish run in the GitHub environment `release`. Its deployment rule allows only tags `v*`. The trusted publishers name the repository `ai-ready-future/jag`, the workflow `release.yml`, and the environment `release`. The environment has no required reviewer, because the user and the sessions use one account | Inferred from the crates.io docs ("Optional environment") and the npm docs (trusted publishers) |
| 10 | Archive form | `jag-<version>-<target>.tar.gz` holds the folder `jag-<version>-<target>/` with `jag` (mode 0755), `README.md`, and `LICENSE`. The checksum file `jag-<version>-<target>.tar.gz.sha256` holds one line: the SHA-256 in hex, two spaces, and the archive name | `docs/adr/0020-name-the-release-archives-and-the-npm-platform-packages-by.md` (accepted: ANSWER human:404pilo, 2026-10-14, hitl) |
| 11 | Crate contents | `Cargo.toml` gets `include = ["/src/**", "/Cargo.toml", "/Cargo.lock", "/README.md", "/LICENSE"]`, `readme`, `keywords`, and `categories`. A check of `cargo package --list` fails on any other path | Inferred from CRITERION-84 and `Cargo.toml` (no `include` today). `src/` holds only `.rs` files, and no code uses `include_str!`. The test build of "Problem" |
| 12 | npm names | The main package: `@airf/jag`. The platform packages: `@airf/jag-linux-x64`, `@airf/jag-linux-arm64`, `@airf/jag-darwin-arm64`, and `@airf/jag-darwin-x64`. Each platform package has one `os` value and one `cpu` value. It has no `libc` value, because the Linux binary is static | ADR-0010. `docs/adr/0020-name-the-release-archives-and-the-npm-platform-packages-by.md` (accepted: ANSWER human:404pilo, 2026-10-14, hitl) |
| 13 | npm package fields | Each `package.json` has `license: MIT`, `repository.url: git+https://github.com/ai-ready-future/jag.git`, and `publishConfig.access: public`. No package has a `preinstall`, `install`, `postinstall`, or `prepare` script. The main package has `bin: {"jag": "bin/jag.js"}` and `engines.node: >=18`. Its `optionalDependencies` name the four platform packages with the exact version. A platform package has no `bin` field | npm docs: provenance needs a `repository` that matches the workflow repository, and a scoped package is not public by default. CRITERION-80: with no install script, the install downloads nothing from GitHub |
| 14 | Launcher | `bin/jag.js` finds the platform package and starts its binary. jag gets the arguments as they are, and the same standard input, output, and error. The launcher gives back the exit code of jag. It sends SIGINT, SIGTERM, SIGHUP, and SIGQUIT on to jag. When jag ends by a signal, the launcher ends by the same signal | Inferred from IF-3, IF-4, CV-4, and CV-6. jag ends by the default action of SIGINT or SIGPIPE (`src/cli/stop.rs`) |
| 15 | Job order and a re-run | The order: build, scan, a draft GitHub Release, crates.io, npm, the checks of the channels, and the publish of the GitHub Release. npm gets the platform packages before the main package. Each publish job skips a version that its channel already has. So "Re-run failed jobs" ends a part release | Inferred: crates.io and npm do not take one version two times. GitHub docs, immutable releases: a draft, then the files, then the publish |
| 16 | Scan of the packages | Before any publish, the job `scan` reads each archive, the `.crate` file, and the five npm tarballs. `check.sh` checks each text line, each string of a binary, and each file name in its text mode. It uses `PRIVATE_PATTERNS` and the gitleaks pin of `private-data.yml`. Exit 1 or 2 stops the release | CRITERION-84. FEATURE-17 spec: "The check of the published packages (CRITERION-84): FEATURE-18". FEATURE-19 spec in pull request #154: the text mode of `check.sh` |
| 17 | Binary check | Linux: `readelf` shows no `INTERP` header and no `NEEDED` entry. macOS: each library that `otool -L` shows is in `/usr/lib/` or `/System/Library/Frameworks/`. On each runner, `jag --version` with an empty environment prints `jag <version>` | CRITERION-78. Deployment-2. Inferred: the platform verifier of `docs/adr/0014-call-the-model-service-with-ureq-and-serde-json-on-std.md` links the macOS Security framework |
| 18 | Tests | Python `unittest` tests for `scripts/release/release.py`, and `node --test` tests for the launcher, in the new job `release-tools` of `ci.yml`. The dry run of each pull request that changes a release file. The job `verify` of each release | Inferred. Q-ar: unit tests now, and the full test when the registries are ready |
| 19 | Precondition | The npm parts and the crates.io parts depend on the user steps U1 to U6, not on a user name. CHORE-02 is U1. See "The user steps before the first tag" | Orchestrator INFO, 2026-10-14: the organization `airf` does not exist yet, nothing is published, and nothing has trusted publishing. crates.io docs. npm docs |
| 20 | First publish of the crate and the npm packages | The user publishes an empty version 0.0.0 of the crate and of each npm package from their own machine (U3). The chores role guides these hand steps. Then the user adds trusted publishing (U4). The first real version tag then publishes each channel with no hand step | ANSWER human:404pilo, 2026-10-14, hitl (the recommended option: an empty first version by hand) |
| 21 | Direct npm publish | The npm trusted publishers allow `npm publish` (`--allow-publish`), not only `npm stage publish` | CRITERION-82: one tag publishes. npm docs: a staged publish waits for the approval of a maintainer with 2FA |
| 22 | Node and npm | Node 24 from `actions/setup-node`, with no cache, in each job that uses npm. The job `npm` needs npm 11.5.1 or later. For a public package from a public repository, npm makes the provenance by itself | npm docs: trusted publishing needs npm 11.5.1 and Node 22.14.0 or later. The example of the npm docs sets `package-manager-cache: false` |
| 23 | crates.io publish | `rust-lang/crates-io-auth-action` gives a token for 30 minutes. Only the step `cargo publish --locked` gets it, through `CARGO_REGISTRY_TOKEN` | crates.io docs: trusted publishing, the setup for GitHub Actions |
| 24 | Release notes | `gh release create --generate-notes`: the merged pull requests since the last tag | Inferred |
| 25 | README | A new section "Install" before "Store your TypeSafe key", with the three channels | METRIC-04. Inferred |
| 26 | macOS signature | No Apple signature and no notarization in this feature. The linker gives each arm64 binary an ad hoc signature. The README tells a user who downloads with a browser how to remove the quarantine mark | ANSWER human:404pilo, 2026-10-14, hitl (no signature now; the README gives the `xattr` command). ADR-0010 names no signature. npm, cargo, and `curl` set no quarantine mark |
| 27 | `jag-bench` | Not in the archives and not in the npm packages. The feature that builds it decides | `docs/adr/0011-build-jag-as-one-cargo-package-with-one-module-per-component.md`: "A second binary, `jag-bench`, comes later in the same package". No roadmap page names it |
| 28 | Secrets, permissions, and cache | `release.yml` uses only the secrets `PRIVATE_PATTERNS` (job `scan`) and `GITHUB_TOKEN`. The workflow default is `contents: read`. A job gets `contents: write` or `id-token: write` only when it needs it. No job uses a cache. Each action has a full commit SHA. Each checkout has `persist-credentials: false` | CRITERION-83. The conventions of `ci.yml` and `private-data.yml`. Inferred: a cache from another run can change what the release builds |
| 29 | When each story is done | STORY-48 to STORY-51 are done after the first release run with each job green, and after U5. A code merge alone does not finish them | CRITERION-77 to CRITERION-84. FEATURE-19 spec decision 25 uses the same form |
| 30 | Story state | STORY-48 to STORY-51 stay `backlog`, with the `spec:` line, because FEATURE-19 is not `done`. The ASKs of decisions 20 and 26 have their ANSWERs (human:404pilo, 2026-10-14) | `docs/adlc.md` rule 11; the ANSWERs of decisions 20 and 26 |
| 31 | LICENSE | The MIT text of GitHub, with the holder "AI Ready Future" and the year 2026, in pull request #181. The release copies it into each archive, the crate, and each npm package | `Cargo.toml`: `license = "MIT"`. Orchestrator ANSWER, 2026-10-14. The holder is inferred: the display name of the organization. The user can change it |
| 32 | Failures of the launcher | The launcher is not jag. When it cannot start jag, it writes one line to the error stream. Then it exits with 127 (no binary) or 126 (the binary does not start). It does this also with `--json` | The architect, 2026-10-14: accepted; system-design.md IF-4 and CV-1 (PR #184) |

## Design

### Files

```
.github/workflows/release.yml       new: the release and the dry run
.github/workflows/ci.yml            changed: the job release-tools
Cargo.toml                          changed: include, readme, keywords, categories
scripts/release/release.py          new: the steps of the release (Python 3.11 or later, standard library only)
scripts/release/test_release.py     new: unittest tests of release.py
scripts/release/first-publish.sh    new: the empty versions of U3 (the user runs it)
scripts/release/README.md           new: the release steps, the user steps, and a re-run
npm/jag/package.json                new: the main package; the release sets each version
npm/jag/bin/jag.js                  new: the launcher
npm/jag/test/launcher.test.js       new: the tests of the launcher (not in the package)
npm/platform/README.md              new: the README of each platform package
README.md                           changed: the section "Install"
```

`LICENSE` comes from pull request #181.

### The platforms

| Target | Runner | npm package | `os` | `cpu` |
|---|---|---|---|---|
| `x86_64-unknown-linux-musl` | `ubuntu-24.04` | `@airf/jag-linux-x64` | `linux` | `x64` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `@airf/jag-linux-arm64` | `linux` | `arm64` |
| `aarch64-apple-darwin` | `macos-15` | `@airf/jag-darwin-arm64` | `darwin` | `arm64` |
| `x86_64-apple-darwin` | `macos-15-intel` | `@airf/jag-darwin-x64` | `darwin` | `x64` |

`release.py` holds this table one time. The job `prepare` writes the matrix
of `build` and `verify` from it, as JSON (`fromJSON`). The Python tests check
that `npm/jag/package.json` names the same four packages. The launcher tests
check that the table of the launcher agrees with `npm/jag/package.json`.

### The workflow `release.yml`

```yaml
on:
  push:
    tags: ['v*']
  pull_request:
    paths:
      - .github/workflows/release.yml
      - scripts/release/**
      - npm/**
      - Cargo.toml
      - Cargo.lock
      - rust-toolchain.toml
  workflow_dispatch:
permissions:
  contents: read
concurrency:
  group: ${{ github.event_name == 'push' && 'release-publish' || format('release-dry-run-{0}', github.ref) }}
  cancel-in-progress: ${{ github.event_name != 'push' }}
```

A comment at the top of the file says: the trusted publishers of crates.io
and npm name this file and the environment `release`. Do not rename them.

A tag release has its own concurrency group. So a dry run never stops a
release that waits.

`publish` is true only for the push of a tag. The jobs, in order:

| Job | Runner | Needs | Runs when | Permissions | Environment |
|---|---|---|---|---|---|
| `prepare` | `ubuntu-24.04` | — | always | `contents: read` | — |
| `npm-main` | `ubuntu-24.04` | `prepare` | always | `contents: read` | — |
| `build` (4) | the table | `prepare`, `npm-main` | always | `contents: read` | — |
| `crate` | `ubuntu-24.04` | `prepare` | always | `contents: read` | — |
| `scan` | `ubuntu-24.04` | `build`, `crate`, `npm-main` | always | `contents: read` | — |
| `github-draft` | `ubuntu-24.04` | `scan` | `publish` | `contents: write` | `release` |
| `crates-io` | `ubuntu-24.04` | `github-draft` | `publish` | `contents: read`, `id-token: write` | `release` |
| `npm` | `ubuntu-24.04` | `crates-io` | `publish` | `contents: read`, `id-token: write` | `release` |
| `verify` (4) | the table | `npm` | `publish` | `contents: read` | — |
| `github-publish` | `ubuntu-24.04` | `verify` | `publish` | `contents: write` | `release` |

No job has both `contents: write` and `id-token: write`. The job `scan` is
the only job that gets `PRIVATE_PATTERNS`, and it has no write permission.

**`prepare`.** Check out with `fetch-depth: 0`. Run `release.py version`,
with `--tag "$GITHUB_REF_NAME"` on the push of a tag. The command reads
`version` from `Cargo.toml` with `tomllib`. It checks the form
`MAJOR.MINOR.PATCH`, and it checks that the tag is `v<version>`. It prints the
version. On the push of a tag, the job also runs
`git merge-base --is-ancestor <commit of the tag> origin/main`. The commit of
the tag comes from `git rev-parse "$GITHUB_REF_NAME^{commit}"`. A failure
stops the release with one line, for example:

```
release: the tag v1.2.3 does not match the version 1.2.2 in Cargo.toml.
```

The outputs: `version`, `publish`, and `matrix`.

**`npm-main`.** `release.py npm-main --version <v> --out <dir>` makes the
main package from `npm/jag/`, the root `README.md`, and `LICENSE`. Then
`npm pack` makes `airf-jag-<v>.tgz`, and
`release.py check-npm-tarball --version <v> <file>` checks it. The job
uploads the file as the artifact `npm-main`.

**`build`.** For each target:

1. On Linux: `sudo apt-get install -y musl-tools`. `ring` compiles C, so the
   musl target needs `musl-gcc`. The step sets `CC_<target>=musl-gcc`, for
   example `CC_x86_64_unknown_linux_musl=musl-gcc`.
2. `rustup target add <target>`. `rust-toolchain.toml` gives the toolchain.
3. `cargo test --locked --release --target <target>`. This runs all tests
   on the target of the archive, also the speed test (`tests/speed.rs`).
4. `cargo build --locked --release --target <target>`.
5. `release.py check-binary --target <target> --version <v> <binary>`. See
   "The binary check".
6. `release.py archive --target <target> --version <v> --binary <binary> --out dist/`.
   See "The archives".
7. `release.py npm-platform --target <target> --version <v> --binary <binary> --out <dir>`,
   then `npm pack`, then `release.py check-npm-tarball`.
8. `release.py local-install --main <npm-main file> --platform <platform file> --version <v>`.
   It unpacks the two tarballs into an empty folder:
   `node_modules/@airf/jag` and `node_modules/@airf/jag-<os>-<cpu>`. Then it
   runs the launcher with `--version` (output `jag <v>`) and with
   `--no-such-option` (exit 2). So the launcher and the binary run as npm
   installs them, before any publish.
9. Upload the archive, its `.sha256` file, and the platform tarball as the
   artifact `build-<target>`.

**`crate`.** `cargo package --list --locked`, then
`release.py check-crate-list <list>`. Then `cargo package --locked`. It
writes `target/package/jag-grep-<v>.crate` and builds the packed files in a
clean folder. Then `cargo publish --dry-run --locked`. It also checks the
metadata that crates.io needs. Upload the `.crate` file as the artifact
`crate`.

**`scan`.** See "The scan of the packages".

**`github-draft`, `crates-io`, `npm`, `verify`, `github-publish`.** See "The
publish jobs and a re-run".

### The version

`Cargo.toml` holds the version one time. A change of the version is a normal
pull request. Change `version` in `Cargo.toml`, and run `cargo check` so that
`Cargo.lock` changes too. Then commit both. `--locked` fails when
`Cargo.lock` does not agree.

In one release, these values agree:

- the tag, and the version in `Cargo.toml` and in `Cargo.lock`
- the output of `jag --version`
- the version of the crate and of the five npm packages
- the four exact versions in the `optionalDependencies` of the main package
- the name of the GitHub Release

`scripts/speed/measure.py` reads the last word of `jag --version`. Keep the
version as the last word. `src/cli.rs` uses the `version` of clap.

### The archives

`release.py archive` writes `jag-<v>-<target>.tar.gz` with Python `tarfile`:

- The folder `jag-<v>-<target>/` (mode 0755), the binary `jag` (mode 0755),
  `README.md` (0644), and `LICENSE` (0644).
- Owner and group 0, and no owner name. The time of each entry is the time
  of the tag commit (`git log -1 --format=%ct`). The gzip header has no file
  name and no time. So the same binary gives the same archive.

Then it writes `jag-<v>-<target>.tar.gz.sha256` with one line:

```
<sha256 hex>  jag-<v>-<target>.tar.gz
```

`sha256sum -c` on Linux and `shasum -a 256 -c` on macOS read this form.

### The crate

`Cargo.toml` changes:

```toml
readme = "README.md"
keywords = ["grep", "search", "cli", "llm"]
categories = ["command-line-utilities", "text-processing"]
include = ["/src/**", "/Cargo.toml", "/Cargo.lock", "/README.md", "/LICENSE"]
```

`release.py check-crate-list` accepts only these paths in the output of
`cargo package --list`:

- each path under `src/`
- `Cargo.toml`, `Cargo.toml.orig`, and `Cargo.lock`
- `README.md` and `LICENSE`
- `.cargo_vcs_info.json`

Any other path gives exit 1 and the number of the line, not the path.

`cargo install jag-grep` builds the binary `jag` (ADR-0011). It needs Rust
1.99 or later (`rust-version`).

### The npm packages

`npm/jag/package.json` is in the repository with the version `0.0.0`:

```json
{
  "name": "@airf/jag",
  "version": "0.0.0",
  "description": "Grep with a model that ranks the matches by how well they answer your question",
  "license": "MIT",
  "repository": { "type": "git", "url": "git+https://github.com/ai-ready-future/jag.git" },
  "homepage": "https://github.com/ai-ready-future/jag#readme",
  "bin": { "jag": "bin/jag.js" },
  "files": ["bin/jag.js"],
  "engines": { "node": ">=18" },
  "optionalDependencies": {
    "@airf/jag-darwin-arm64": "0.0.0",
    "@airf/jag-darwin-x64": "0.0.0",
    "@airf/jag-linux-arm64": "0.0.0",
    "@airf/jag-linux-x64": "0.0.0"
  },
  "publishConfig": { "access": "public" }
}
```

`release.py npm-main` copies `bin/jag.js`, the root `README.md`, and
`LICENSE`. It sets `version` and each value of `optionalDependencies` to the
release version. The README of the repository is then also the npm page.

`release.py npm-platform` writes the folder `jag-<os>-<cpu>/`:

```json
{
  "name": "@airf/jag-linux-x64",
  "version": "<v>",
  "description": "The jag binary for Linux x64. Install @airf/jag, not this package.",
  "license": "MIT",
  "repository": { "type": "git", "url": "git+https://github.com/ai-ready-future/jag.git" },
  "homepage": "https://github.com/ai-ready-future/jag#readme",
  "os": ["linux"],
  "cpu": ["x64"],
  "files": ["bin/jag"],
  "publishConfig": { "access": "public" }
}
```

The folder also holds `bin/jag` (mode 0755), `LICENSE`, and `README.md`. The
README comes from `npm/platform/README.md`, with the platform filled in.

`release.py check-npm-tarball` reads the tarball with `tarfile`. It checks:

- the name and the version
- the list of files
- no `preinstall`, `install`, `postinstall`, or `prepare` script
- the `repository.url`
- mode 0755 for `package/bin/jag` in a platform package

npm keeps the mode of a file, and the launcher needs the execute bit.

### The launcher (`npm/jag/bin/jag.js`)

The launcher is CommonJS for Node 18 or later. It uses only the built-in
modules `node:child_process`, `node:os`, and `node:path`, and no network
module. It exports `packageName` and `messages` for the tests. It runs only
when `require.main === module`.

1. Name: the launcher finds the package in the table of the four pairs of
   `process.platform` and `process.arch` ("The platforms"). For another
   pair, it writes this line and exits with 127:

   ```
   jag: npm has no jag binary for <platform> <arch>. jag runs on Linux x64 and arm64, and on macOS arm64 and x64. To build jag from source, run: cargo install --locked jag-grep
   ```

2. Path: `require.resolve('<package>/bin/jag')`. When npm did not install
   the package, the launcher writes this line and exits with 127:

   ```
   jag: the package <package> is not installed. npm does not install it with --omit=optional or --no-optional. Install @airf/jag again without these options.
   ```

3. Start: `spawn(path, process.argv.slice(2), { stdio: 'inherit' })`. jag
   gets the arguments as they are, the same environment, and the same
   standard input, output, and error. So jag itself sees the terminal or the
   pipe ("In a terminal and in a pipe" in the README), and `--setup` gets
   the terminal. When the start fails, the launcher writes this line:

   ```
   jag: cannot start <path>: <error code>. Install @airf/jag again.
   ```

   Then it exits with 127 for `ENOENT`, else with 126.
4. Signals: for SIGINT, SIGTERM, SIGHUP, and SIGQUIT, a listener sends the
   signal on to jag. An agent that stops the launcher with SIGTERM then also
   stops jag. With Ctrl-C in a terminal, jag can get SIGINT two times. The
   terminal sends one, and the launcher sends one. jag ends at the first one
   (`src/cli/stop.rs`), so the second one has no effect.
5. End: when jag exits with a code, the launcher exits with the same code
   (IF-4). jag can also end by a signal (CV-4: SIGINT, or SIGPIPE on a closed
   pipe). Then the launcher removes its listeners and sends the same signal
   to itself. Node ignores SIGPIPE, so for SIGPIPE the launcher exits with
   `128 + 13` (141).

The launcher writes nothing else. It never writes an argument or a variable
(CV-6). Its own failures use 126 and 127 (decision 32). A shell uses these
codes when it cannot run a command. IF-4 says that they mean that jag did
not start, so an agent does not read them as a jag result. With `--json`, a
failure of the launcher gives no document on standard output. IF-5 says that
such an output means that jag stopped.

### The scan of the packages

The job `scan` downloads the artifacts of `build`, `crate`, and `npm-main`.
These are 14 files: four archives, four `.sha256` files, one `.crate` file,
and five npm tarballs. It downloads gitleaks with the same URL and SHA-256
as `private-data.yml`.

`release.py texts --text <tmp>/text --index <tmp>/index <file>...` reads
each file with `tarfile`, and the `.sha256` files as text. For member number
`k` of each file:

- The name of the member gives one line. Its index line is
  `<file> member <k> name`.
- A member with a zero byte in its first 8192 bytes is a binary, as in the
  history mode of FEATURE-19. Each run of 4 or more printable ASCII bytes
  gives one line. These bytes are `0x20` to `0x7e`, and tab. The index line
  is `<file> member <k> byte <offset>`.
- Another member gives each of its lines. The text is UTF-8, and a bad byte
  becomes U+FFFD. The index line is `<file> member <k> line <n>`.

`<file>` is the name of the artifact file, which holds only the version and
the target. An index line never holds a member path or a value. Then:

```yaml
- name: Scan the packages
  env:
    PRIVATE_PATTERNS: ${{ secrets.PRIVATE_PATTERNS }}
    GITLEAKS: ${{ runner.temp }}/gitleaks/gitleaks
    TEXT_FILE: ${{ runner.temp }}/scan/text
    TEXT_INDEX: ${{ runner.temp }}/scan/index
  run: scripts/private-data/check.sh
```

`check.sh` runs both configs: the key rules of gitleaks and the private
patterns. A finding gives `<index line>: <RuleID>` and exit 1. Exit 1 or 2
stops the release before any publish.

The job `crates-io` publishes from the same commit, so `cargo publish` packs
the same files as the scanned `.crate` file. The job `npm` publishes the
scanned tarballs.

A binary holds the paths of the source files of its crates. The runners
build the binaries, so the paths name the runner (`/home/runner/` or
`/Users/runner/`), not a person. The release never uses a binary from the
machine of a person.

### The binary check

`release.py check-binary`:

- Linux: `readelf -lW` must show no `INTERP` header, and `readelf -dW` must
  show no `(NEEDED)` entry. The binary is then fully static. It needs no C
  library and no loader of the machine.
- macOS: each line after the first of `otool -L` must name a file in
  `/usr/lib/` or `/System/Library/Frameworks/`. Each macOS has them. The
  platform verifier (ADR-0014) links the Security framework.
- On each runner: `jag --version` with an empty environment must exit 0 and
  print `jag <v>`.

A Linux machine needs the CA certificates of the system for a call to the
model service (ADR-0014). Certificates are data, not a runtime or a program.
`--version` and plain grep need none.

### The publish jobs and a re-run

Each publish job first asks its channel for the version. When the channel
has it, the job skips the publish and ends green. So "Re-run failed jobs"
ends a release that stopped part way. The re-run uses the artifacts of the
same run.

- **`github-draft`.** A published release for the tag: skip. A draft for the
  tag: `gh release upload v<v> <8 files> --clobber`. Else:
  `gh release create v<v> --draft --verify-tag --title "jag <v>" --generate-notes <8 files>`.
  The 8 files are the 4 archives and the 4 `.sha256` files. `GH_TOKEN` is
  `github.token`.
- **`crates-io`.** Check out the tag. First,
  `release.py exists --registry crates --name jag-grep --version <v>` asks
  `https://crates.io/api/v1/crates/jag-grep/<v>`, with a `User-Agent`
  header. When the version does not exist, the job runs
  `rust-lang/crates-io-auth-action`, then `cargo publish --locked`.
  `CARGO_REGISTRY_TOKEN` comes from the output `token` of the action, in that
  step only.
- **`npm`.** `actions/setup-node` with `node-version: 24`,
  `registry-url: https://registry.npmjs.org`, and
  `package-manager-cache: false`. `release.py check-npm-version "$(npm --version)"`
  needs 11.5.1 or later. Then the job publishes the four platform tarballs,
  and last the main tarball. For each one:
  `release.py exists --registry npm --name <name> --version <v>`, else
  `npm publish <file>`. No step sets `NODE_AUTH_TOKEN`, so npm uses trusted
  publishing. For a tarball file, `npm publish` takes the same path as for a
  folder, with the OIDC exchange and the provenance.
- **`verify`.** On each runner of the table, with an empty npm cache:
  1. `npm install @airf/jag@<v> --loglevel http` in an empty folder. Then
     `release.py check-npm-install --log <log> --dir <folder> --target <target> --version <v>`
     checks three points. Each `http fetch` line of the log names a URL in
     `https://registry.npmjs.org/`, and the log has 2 such lines or more.
     `node_modules/@airf` holds only `jag` and the package of the runner.
     `node_modules/.bin/jag --version` prints `jag <v>`.
  2. `npx -y @airf/jag@<v> --version` prints `jag <v>`.
     `npx -y @airf/jag@<v> --no-such-option` exits with 2.
     `npx -y @airf/jag@<v> jag-release-check <empty folder>` exits with 1.
  3. On `ubuntu-24.04` only:
     `cargo install --locked jag-grep --version <v> --root <tmp>`. Then
     `<tmp>/bin/jag --version` prints `jag <v>`.

  A new version can take some time to show on a registry. Each step tries
  again up to 5 times, with 60 seconds between the tries.
- **`github-publish`.** First `gh release download v<v>`, and
  `release.py check-release --dir <dir> --version <v>`. The check: the
  release has exactly the 8 files, and each `.sha256` file agrees with its
  archive. Then, for a draft: `gh release edit v<v> --draft=false --latest`.
  A published immutable release cannot change its files, so the check runs
  on the draft. When the check fails, the release stays a draft.

When a job fails after a channel published, the version on that channel
stays. A fix that needs new code needs a new patch version. Then the user
can mark the bad version: `cargo yank --version <v> jag-grep`, and
`npm deprecate @airf/jag@<v> "<reason>"`. `scripts/release/README.md` gives
these steps.

### The user steps before the first tag

The chores role guides the user through these steps. No step names an npm
user. Each step uses the account that the user logs in with.

- **U1 (CHORE-02).** The npm organization `airf` exists, and the user owns
  it. Check: `npm org ls airf` shows the user as the owner. ADR-0010: a
  person must confirm this before the first release.
- **U2. The repository, after FEATURE-19.**
  - The environment `release`, with a deployment rule for the tags `v*`
    only.
  - A tag ruleset for `refs/tags/v*`, with the rules "update" and "deletion"
    and no bypass.
  - Immutable releases on.
  - Check: `gh api repos/ai-ready-future/jag/environments/release`, and
    `gh api repos/ai-ready-future/jag/rulesets`.
- **U3. The first publish (decision 20).** On the
  machine of the user, `scripts/release/first-publish.sh` makes an empty
  version 0.0.0 of the crate `jag-grep` and of the five npm packages. It
  works in a new temporary folder.
  - Each one holds only `package.json` or `Cargo.toml`, `LICENSE`, and a
    README with "This version holds no program. Install a later version."
  - The crate has an empty `src/lib.rs`.
  - The script runs `cargo publish` with a crates.io token that the user
    makes for this step only.
  - It runs `npm publish` for each npm package, and npm asks for the 2FA
    code.
  - The script reads no key file and writes no token.
- **U4. The trusted publishers.**
  - crates.io: open the crate `jag-grep`, then Settings, then Trusted
    Publishing. Add a GitHub publisher with these values: owner
    `ai-ready-future`, repository `jag`, workflow `release.yml`, and
    environment `release`.
  - npm: use npm 11.15.0 or later, with 2FA on. Run this command for each of
    the five packages:
    `npm trust github <package> --file release.yml --repo ai-ready-future/jag --env release --allow-publish`.
- **U5. Close the token routes.**
  - crates.io: in the settings of the crate `jag-grep`, turn on "Trusted
    Publishing only". Then revoke the crates.io token of U3.
  - npm: in the settings of each package, under Publishing access, choose
    "Require two-factor authentication and disallow tokens".
  - Check: `gh secret list` and `gh secret list --env release` show no
    publish token.
- **U6. The first tag, in the 2 days after U4.** A new npm trusted publisher
  expires when no publish uses it in 2 days. Before the tag, the dry run on
  `main` must be green. Then:

  ```
  $ git tag -a v<v> -m "jag <v>" <commit of main>
  $ git push origin v<v>
  ```

After the first release, the user can mark the empty versions:
`cargo yank --version 0.0.0 jag-grep`, and
`npm deprecate <package>@0.0.0 "This version holds no program."`.

### README

A new section "Install", after the first paragraph and before "Store your
TypeSafe key":

````markdown
## Install

jag runs on Linux (x86_64 and arm64) and on macOS (arm64 and x86_64). Each
way gives the same version.

With npm (Node.js 18 or later):

```
$ npm install -g @airf/jag
```

Or run jag with no install:

```
$ npx @airf/jag --help
```

With cargo (Rust 1.99 or later). This builds jag from source in some
minutes:

```
$ cargo install --locked jag-grep
```

From GitHub Releases: download the archive for your system and its
`.sha256` file from the latest release. Then check the archive, open it, and
move `jag` to a folder in your `PATH`:

```
$ sha256sum -c jag-<version>-<target>.tar.gz.sha256        # Linux
$ shasum -a 256 -c jag-<version>-<target>.tar.gz.sha256    # macOS
$ tar -xzf jag-<version>-<target>.tar.gz
$ mv jag-<version>-<target>/jag ~/.local/bin/
```

The targets are `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`,
`aarch64-apple-darwin`, and `x86_64-apple-darwin`.

On macOS, a browser marks each file that it downloads, and macOS then stops
`jag`. Download with `curl -LO`, or remove the mark:
`xattr -d com.apple.quarantine jag`.

On Linux, a question to the model needs the CA certificates of the system
(the package `ca-certificates`).
````

### Acceptance criteria and their tests

| Criterion | Test |
|---|---|
| CRITERION-77 | `test_release.py`: the archive has the folder, the three files, and their modes. The `.sha256` line agrees with the archive. Release run: the job `github-publish` finds exactly the 4 archives and the 4 `.sha256` files, and each check passes |
| CRITERION-78 | `test_release.py`: the parsers of `readelf` and `otool` with sample output. A static binary passes. An `INTERP` header, a `NEEDED` entry, or a library in `/opt/homebrew/` fails. Dry run and release run: `release.py check-binary` on each of the 4 runners, with `jag --version` in an empty environment |
| CRITERION-79 | Release run: the job `verify` on `ubuntu-24.04` runs `cargo install --locked jag-grep --version <v>`. Then `jag --version` prints `jag <v>`. Dry run: `cargo publish --dry-run --locked` builds the packed crate |
| CRITERION-80 | `launcher.test.js` and `test_release.py`: the 5 packages have no install script, and the launcher loads no network module. Dry run: `release.py local-install` on each of the 4 runners. Release run: in the job `verify`, `npx -y @airf/jag@<v>` runs jag on each of the 4 runners. The npm log names only `https://registry.npmjs.org/` |
| CRITERION-81 | `test_release.py`: each platform package has one `os` value and one `cpu` value. The main package has the four exact versions. Release run: in the job `verify`, `node_modules/@airf` holds only `jag` and the package of the runner, on each of the 4 runners |
| CRITERION-82 | `test_release.py`: `release.py version` with a tag that agrees, a tag that does not agree, and a pre-release version. `npm-main` and `npm-platform` write the version in each place. Release run: `prepare`, `check-binary`, `verify`, and `github-publish` each compare the version |
| CRITERION-83 | `test_release.py` reads `release.yml` as text. It names no secret other than `PRIVATE_PATTERNS` and `GITHUB_TOKEN`. Only the jobs `crates-io` and `npm` have `id-token: write`. No step sets `NODE_AUTH_TOKEN`. U5: the checks of the secret lists and of the two registry settings. Release run: the jobs `crates-io` and `npm` publish with no stored token |
| CRITERION-84 | `test_release.py`: `release.py texts` with invented files. A text member, a binary member, and a member name each give their index lines. No index line holds a member path. `release.py check-crate-list` fails for an invented list with `docs/x.md`. Dry run and release run: the job `scan` exits 0 before any publish |

The launcher tests (`node --test npm/jag/test/launcher.test.js`) use a fake
platform package in a temporary `node_modules`. The fake binary is a small
script. The tests cover these cases:

- the name for each of the four pairs, and an unknown pair (127 and the
  message)
- a missing package (127)
- arguments with spaces, an empty argument, and `--`
- the exit codes 0, 1, 2, 7, and 130
- an end by SIGINT: the launcher ends by SIGINT
- an end by SIGPIPE: the launcher ends by SIGPIPE or exits with 141
- SIGTERM to the launcher reaches the fake binary
- a binary with no execute bit (126)
- the table of the launcher agrees with `npm/jag/package.json`

The Python tests read `release.yml` as text, because the standard library of
Python has no YAML parser. A job block starts at a line with two spaces, a
name, and a colon, under `jobs:`.

The job `release-tools` of `ci.yml` runs on each pull request:

```
python3 -I -m unittest discover -s scripts/release -p 'test_*.py'
node --test npm/jag/test/launcher.test.js
```

Its matrix: `ubuntu-24.04` with Node 18, `ubuntu-24.04` with Node 24, and
`macos-15` with Node 24. It is not a required check.

## Build order

1. Preconditions: pull request #181 (LICENSE) is merged. FEATURE-19 is done:
   the public repository, the text mode of `check.sh`, and the event policy.
2. STORY-48:
   - `release.py`: `version`, `check-binary`, `archive`, `texts`,
     `check-crate-list`, and `check-release`.
   - The jobs `prepare`, `build` (steps 1 to 6, and 9), `crate`, and `scan`.
   - The job `release-tools`, and the GitHub Releases part of the README.
   - Test: the unit tests, and the dry run of the pull request, with each
     job green on the 4 runners.
3. STORY-49: the `include` list and the other fields of `Cargo.toml`,
   `cargo publish --dry-run` in the job `crate`, and the cargo part of the
   README. Test: the dry run.
4. STORY-50:
   - `npm/jag/`, `npm/platform/README.md`, and the job `npm-main`.
   - Steps 7 and 8 of `build`, the launcher tests, and the npm part of the
     README.
   - Test: the unit tests, the launcher tests, and the dry run.
5. STORY-51:
   - The jobs `github-draft`, `crates-io`, `npm`, `verify`, and
     `github-publish`.
   - `release.py`: `exists`, `check-npm-version`, and `check-npm-install`.
   - `first-publish.sh` and `scripts/release/README.md`.
   - Test: the unit tests, and the dry run. The publish jobs do not run in a
     dry run.
6. The user steps U1 to U5, with the chores role (decision 20).
7. U6: the first version tag. The user decides the version and the day
   (SCOPE-01). A green run proves CRITERION-77 to CRITERION-84 (decision
   29).

## Assumptions

- Decision 2 (musl): a Rust musl target links statically by default, and
  `ring` is the only crate that compiles C (`Cargo.lock`, relay:scout). The
  test build of "Problem" covers `x86_64-unknown-linux-musl` only.
- Decision 3 (runners): from the runner table of GitHub and the runner image
  pages on 2026-10-14. `ubuntu-24.04-arm` has Rust 1.99.0, Python 3.12.3,
  Node 22, and binutils. `musl-tools` gives `musl-gcc` on arm64 too (not
  checked). The first dry run checks it.
- Decisions 5 to 9, 11, 13 to 18, 24, 25, 27, and 28: inferred, with the
  sources in the table.
- Decision 11: `src/` holds only `.rs` files, and no code uses
  `include_str!` or `include_bytes!` (`src/`, 2026-10-14). A later file that
  the code reads at build time must be under `src/`.
- Decision 14: jag ends by the default action of SIGINT or SIGPIPE
  (`src/cli/stop.rs`). `--version` prints `jag <version>` (`src/cli.rs`,
  the `version` of clap).
- Decision 31: the holder "AI Ready Future" is the display name of the
  organization `ai-ready-future`.
- The job `npm`: for a tarball file, `npm publish` takes the same path as
  for a folder. It packs the file with `pacote` and sends it with
  `libnpmpublish`, with the provenance (npm CLI, `lib/commands/publish.js`,
  through context7 `/npm/cli`).
- `npm install --loglevel http` writes one `http fetch` line for each
  request. The check needs 2 such lines or more, so a changed log form fails
  the check. The check does not pass with no lines.
- An empty version 0.0.0 obeys the crates.io usage policy. The policy is
  about a crate that "exists only to reserve a name for a prolonged period
  of time". U6 comes in the 2 days after U4, so the empty version stays
  alone for a short time only.
- `cargo publish --dry-run --locked` needs no token (the test build of
  "Problem").

## Risks

- `ring` does not build for `aarch64-unknown-linux-musl`. Signal: the step
  `cargo test` of `build` fails in the build script of `ring`. Mitigation:
  build that target with `cargo-zigbuild` or `cross`.
- GitHub removes the Intel macOS runners. Signal: the job on
  `macos-15-intel` does not start. Mitigation: build `x86_64-apple-darwin`
  on the arm64 runner with `--target`, and run its checks with Rosetta 2.
- The memory allocator of musl makes jag slower on Linux. Signal: the speed
  test fails in `cargo test --target <musl target>`. Mitigation: another
  allocator for the musl targets, as a later decision.
- A key rule of gitleaks matches a string of a binary that is not a key.
  Signal: the job `scan` exits 1 with a `byte` index line and a rule id such
  as `generic-api-key`. Mitigation: look at the string in a local run, never
  in the log. If it is not a key, add an allow rule to the gitleaks config of
  `check.sh` in a pull request. FEATURE-17 owns that config.
- The text mode of `check.sh` comes from the FEATURE-19 spec in pull request
  #154, which is not merged. Signal: the merged FEATURE-19 spec or
  `check.sh` uses another form of `TEXT_FILE` or `TEXT_INDEX`. Mitigation:
  `release.py texts` and the job `scan` follow the merged form.
- Pull request #170 (FEATURE-13) also changes `README.md` and `Cargo.toml`.
  Signal: a merge conflict in the build pull request of STORY-48 or
  STORY-49. Mitigation: the build merges `main` first and keeps both
  changes.
- A release stops part way, after a channel published. Signal: a red
  publish job. Mitigation: "Re-run failed jobs". For a fix with new code: a
  new patch version, and `cargo yank` and `npm deprecate` for the bad one.
- An npm trusted publisher expires before the first tag. Signal: the job
  `npm` fails with an authentication error at the first release. Mitigation:
  the user makes the trusted publisher again (U4), then clicks "Re-run
  failed jobs".
- An npm package gets no provenance. Signal: the npm page of the version
  shows no provenance. Mitigation: the job `npm` unpacks each scanned
  tarball and publishes the folder. A check compares the file list of
  `npm pack --dry-run` with the tarball.
- CHORE-02 stays open. Signal: chores reports
  U1 open. Mitigation: no first tag. The dry run does not need the
  registries.
- npm loses the execute bit of `bin/jag`. Signal: `check-npm-tarball` or
  `local-install` fails, or `verify` gets 126. Mitigation: the checks run
  before any publish.
- A session pushes a `v*` tag with no TASK. Signal: a release run that no
  TASK names. Mitigation: the user cancels the run before the publish jobs,
  which wait for the build and the scan. The user can add a required
  reviewer to the environment `release`.
- On a Linux machine with no CA certificates, the model calls fail. Signal:
  an issue with a TLS error. Mitigation: the README says that a question to
  the model needs `ca-certificates`.
- A browser download on macOS gets the quarantine mark. Signal: a user
  reports that macOS stops `jag`. Mitigation: the README text. A later
  feature can add an Apple signature and notarization, with an Apple
  account.
- A new version shows late on a registry. Signal: `exists` says "no" right
  after a publish, and the publish of a re-run fails because the version
  exists. Mitigation: wait some minutes, then click "Re-run failed jobs".
- GitHub Actions is blocked for billing reasons (2026-10-14). Signal: each
  job fails with "The job was not started because recent account payments
  have failed". Mitigation: the release runs in the public repository,
  where the standard runners are free.
- A fork pull request that changes a release file gets no secret. Signal:
  its job `scan` stops with exit 2. Mitigation: none is necessary. The check
  is not required, and a maintainer runs the dry run after the merge.

## Out of scope

- Windows and other platforms (deployment-2).
- `jag-bench` in the archives and the npm packages (decision 27).
- An Apple signature and notarization (decision 26).
- Homebrew, apt, `cargo binstall` metadata, and other package managers.
- A test of yarn and pnpm. They also read `os` and `cpu`, but no test covers
  them.
- The version and the day of the first release: the user (SCOPE-01).
- The agent instructions to copy (CAPABILITY-11): another feature.
- The history scan and the clean history: FEATURE-19.
