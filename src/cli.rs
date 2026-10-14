//! C-cli: option parsing, dispatch, the search flow, and the exit code.

mod grep;
mod stop;

use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::builder::TypedValueParser;
use clap::error::ErrorKind;
use clap::{CommandFactory, FromArgMatches, Parser};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;

use crate::check;
use crate::config::{
    self, ConfigError, ConfigFile, DEFAULT_SIZE_LIMIT, Env, RealFs, SettingOptions, SettingSource,
    SizeLimit, TimeSetting, TtyPrompter, cost_limit, find_key, model_settings, normalize_base_url,
    parse_cost_limit, parse_seconds, parse_size_limit, size_limit,
};
use crate::failure::Failure;
use crate::output::progress::Progress;
use crate::output::{
    Paint, Summary, is_no_good_answer, size_notice, write_document, write_lines, write_text,
};
use crate::prerank;
use crate::scorer::{ScoreError, ScoreSettings, probe, rank, score};
use crate::search::input::{Input, no_path_input};
use crate::search::snippet::{
    ContextLines, Snippet, line_windows, match_texts, plan_windows, read_snippets,
};
use crate::search::{
    FileFilter, GlobKind, ScanOptions, ScanReport, WalkWarning, scan, scan_current_folder,
    scan_input,
};
use grep::{MatchOptions, Report, build_matcher};
use stop::Stop;

/// The longest question, in UTF-8 bytes (decision 30).
const MAX_QUESTION_BYTES: usize = 12_000;

/// The one line for each argument error. It never holds a value from the
/// command line, because a value can be a key (CV-6, decision 26).
const USAGE_ERROR: &str = "jag: the command line is not valid. Run jag --help to see the usage.";

/// The options that `--check`, `--setup`, and `--forget-key` refuse (FEATURE-13,
/// decision 6).
const MODES: [&str; 3] = ["setup", "forget_key", "check"];

/// Grep with a model that ranks the matches by how well they answer your question.
#[derive(Parser, Debug)]
#[command(
    name = "jag",
    version,
    arg_required_else_help = true,
    override_usage = "jag [OPTIONS] TERM [PATH...]\n       jag [OPTIONS] --ask QUESTION TERM [PATH...]\n       jag [OPTIONS] -e TERM ... [PATH...]\n       jag --check [--timeout SECONDS] [--retry-wait SECONDS] [--base-url URL] [--model NAME]\n       jag --setup\n       jag --forget-key"
)]
struct Cli {
    /// Write one JSON document for an agent, in place of text
    #[arg(long, conflicts_with_all = ["setup", "forget_key", "check"])]
    json: bool,

    /// When to color the matches: never, always, or auto (the default: only in a terminal)
    #[arg(
        long,
        value_name = "WHEN",
        alias = "colour",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "auto",
        value_parser = ["never", "always", "auto"],
        overrides_with = "color",
        conflicts_with_all = ["setup", "forget_key", "check"]
    )]
    color: Option<String>,

    /// Match upper case and lower case letters as the same
    #[arg(short = 'i', long = "ignore-case", conflicts_with_all = MODES)]
    ignore_case: bool,

    /// Match only whole words
    #[arg(short = 'w', long = "word-regexp", conflicts_with_all = MODES)]
    word: bool,

    /// Read the term as fixed text, not as a regular expression
    #[arg(short = 'F', long = "fixed-strings", conflicts_with_all = MODES)]
    fixed: bool,

    /// Use TERM as the term. With more than one -e, a line matches one of the terms
    #[arg(short = 'e', long = "regexp", value_name = "TERM", allow_hyphen_values = true, conflicts_with_all = MODES)]
    terms: Vec<String>,

    /// Show NUM lines after each match
    #[arg(short = 'A', long = "after-context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "after", conflicts_with_all = MODES)]
    after: Option<String>,

    /// Show NUM lines before each match
    #[arg(short = 'B', long = "before-context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "before", conflicts_with_all = MODES)]
    before: Option<String>,

    /// Show NUM lines before and after each match
    #[arg(short = 'C', long = "context", value_name = "NUM", allow_hyphen_values = true, overrides_with = "context", conflicts_with_all = MODES)]
    context: Option<String>,

    /// Show only the path of each file with a match (not with --ask or --json)
    #[arg(short = 'l', long = "files-with-matches", conflicts_with_all = MODES)]
    files_with_matches: bool,

    /// Show only the count of matching lines of each file (not with --ask or --json)
    #[arg(short = 'c', long = "count", conflicts_with_all = MODES)]
    count: bool,

    /// Write nothing to standard output: give only the exit code (not with --ask or --json)
    #[arg(short = 'q', long = "quiet", alias = "silent", conflicts_with_all = MODES)]
    quiet: bool,

    /// Search only the files whose name matches GLOB
    #[arg(long, value_name = "GLOB", conflicts_with_all = MODES)]
    include: Vec<String>,

    /// Do not search the files whose name matches GLOB
    #[arg(long, value_name = "GLOB", conflicts_with_all = MODES)]
    exclude: Vec<String>,

    /// The globs of --include and --exclude, in the order of the command line
    #[arg(skip)]
    globs: Vec<(GlobKind, String)>,

    /// Check that jag is ready: the key, the service, and the key file
    #[arg(long, conflicts_with_all = ["setup", "forget_key", "cost_limit", "term", "ask", "paths"])]
    check: bool,

    /// Store your API key, with no echo: the TypeSafe key, or the gateway key when JAG_BASE_URL or config.base-url names a gateway
    #[arg(long, conflicts_with_all = ["forget_key", "term", "ask", "paths"])]
    setup: bool,

    /// Remove the stored key: the TypeSafe key, or the gateway key when JAG_BASE_URL or config.base-url names a gateway
    #[arg(long = "forget-key", conflicts_with_all = ["term", "ask", "paths"])]
    forget_key: bool,

    /// Seconds to wait for one model call, from 1 to 60 (default 4)
    #[arg(long, value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
    timeout: Option<String>,

    /// Seconds to wait before the one retry, from 1 to 60 (default 1)
    #[arg(long = "retry-wait", value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
    retry_wait: Option<String>,

    /// The address of the model service or of an AI gateway (default https://api.typesafe.ai/v1)
    #[arg(long = "base-url", value_name = "URL", conflicts_with_all = ["setup", "forget_key"])]
    base_url: Option<String>,

    /// The model name to send (default jev-1.13.0)
    #[arg(long, value_name = "NAME", conflicts_with_all = ["setup", "forget_key"])]
    model: Option<String>,

    /// The most results that the model scores, from 1 to 2,000 (default 200)
    #[arg(long = "max-scored", value_name = "COUNT", conflicts_with_all = ["setup", "forget_key"])]
    cost_limit: Option<String>,

    /// The most characters that jag writes to standard output, 1,000 or more (default 20,000)
    #[arg(long = "max-chars", value_name = "COUNT", conflicts_with_all = ["check", "setup", "forget_key"])]
    size_limit: Option<String>,

    /// Turn off the size limit on standard output
    #[arg(long = "no-max-chars", conflicts_with_all = ["size_limit", "check", "setup", "forget_key"])]
    no_size_limit: bool,

    /// What you look for: the model ranks the matches by this question. With no --ask, jag works as grep: no model call, no cost, and no key
    #[arg(long, value_name = "QUESTION")]
    ask: Option<String>,

    /// The regular expression that the lines must match
    #[arg(value_name = "TERM")]
    term: Option<OsString>,

    /// The files and folders to search. With no path, jag searches the text of a pipe or the current folder
    // The default parser of `PathBuf` refuses an empty path with the fixed line
    // of an argument error. This parser keeps it for the failure of
    // FEATURE-14 decision 8.
    #[arg(
        value_name = "PATH",
        value_parser = clap::builder::OsStringValueParser::new().map(PathBuf::from)
    )]
    paths: Vec<PathBuf>,
}

impl Cli {
    /// The last `--color` wins (clap keeps it). No option is `auto`.
    fn color_when(&self) -> ColorWhen {
        match self.color.as_deref() {
            Some("never") => ColorWhen::Never,
            Some("always") => ColorWhen::Always,
            _ => ColorWhen::Auto,
        }
    }
}

/// The value of `--color` (decision 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorWhen {
    Never,
    Always,
    Auto,
}

/// True when the text output gets color (decisions 5 and 6).
fn color_on(when: ColorWhen, stdout_is_terminal: bool, term: Option<&OsStr>) -> bool {
    match when {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => stdout_is_terminal && term.is_some_and(|term| term != "dumb"),
    }
}

/// What the search flow shares with a stop and with the terminal.
struct Context<'a> {
    mode: Mode,
    stop: &'a Stop,
    progress: &'a Progress,
    stdout_is_terminal: bool,
}

/// Remove the progress line on each path out of the scoring step.
struct ClearOnDrop<'a>(&'a Progress);

impl Drop for ClearOnDrop<'_> {
    fn drop(&mut self) {
        self.0.clear();
    }
}

/// Wait for the end of the process. A stop ends it, and the search flow
/// writes nothing meanwhile (decision 14).
fn wait_for_the_end() -> ! {
    loop {
        std::thread::park();
    }
}

/// Parse the command line, run the command, and give the exit code.
pub fn run() -> ExitCode {
    let cli = match parse() {
        Ok(cli) => cli,
        Err(code) => return code,
    };
    let vars = |name: &str| std::env::var_os(name);
    let home = std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from);
    let env = Env { vars: &vars, home };
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    let mode = if cli.json { Mode::Json } else { Mode::Text };
    let mut summary = Summary::default();
    // The size limit that jag knows when a failure comes (STORY-26).
    let mut known_limit = Some(DEFAULT_SIZE_LIMIT);

    let result = if cli.setup {
        config::setup(
            &env,
            &RealFs,
            &mut TtyPrompter,
            &mut stdout,
            io::stdin().is_terminal(),
        )
        .map(|()| 0)
        .map_err(Failure::from)
    } else if cli.forget_key {
        config::forget_key(&env, &RealFs, &mut stdout)
            .map(|()| 0)
            .map_err(Failure::from)
    } else if cli.check {
        check(&cli, &env, &mut stdout, &mut stderr)
    } else {
        let progress = Arc::new(Progress::new(
            Box::new(io::stderr()),
            mode == Mode::Text && io::stderr().is_terminal(),
        ));
        let stop = Arc::new(Stop::new(mode, Arc::clone(&progress)));
        // With no handler, the default action of SIGINT stays (decision 11).
        let _ = stop::install(&stop);
        stop::watch_pipe(&stop);
        let context = Context {
            mode,
            stop: &stop,
            progress: &progress,
            stdout_is_terminal: io::stdout().is_terminal(),
        };
        let result = search(&cli, &env, &context, &mut summary, &mut stdout, &mut stderr);
        claim_or_wait(&stop);
        known_limit = stop.size_limit();
        result
    };
    finish(
        result,
        mode,
        &mut summary,
        known_limit,
        &mut stdout,
        &mut stderr,
    )
}

/// The form of the output (DECISION-27). Only `--json` gives `Json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Text,
    Json,
}

/// The long options whose value can start with `-` (decisions 10 and 12).
const HYPHEN_VALUE_OPTIONS: [&str; 4] = [
    "--regexp",
    "--after-context",
    "--before-context",
    "--context",
];

/// The letters of the short options whose value can start with `-`.
const HYPHEN_VALUE_LETTERS: [char; 4] = ['e', 'A', 'B', 'C'];

/// True when clap reads `--json` as the option (FEATURE-13 spec decision 38).
/// The words go from left to right, as clap reads them. A word that is the
/// value of `-e`, `-A`, `-B`, `-C`, or their long names is not an option, also
/// when it is `--json` or `--`. Else, after `--`, `--json` is a term or a
/// path, not the option.
fn json_requested(args: impl IntoIterator<Item = OsString>) -> bool {
    // The next word is the value of the option before it.
    let mut value_next = false;
    for arg in args.into_iter().skip(1) {
        if value_next {
            value_next = false;
            continue;
        }
        let Some(word) = arg.to_str() else {
            continue;
        };
        if word == "--" {
            return false;
        }
        if word == "--json" {
            return true;
        }
        if HYPHEN_VALUE_OPTIONS.contains(&word) {
            value_next = true;
        } else if word.starts_with('-') && !word.starts_with("--") {
            // A cluster of short options: a letter that takes a value takes
            // the rest of the word, or the next word when no letter follows.
            let letters = &word[1..];
            if let Some(place) = letters.find(HYPHEN_VALUE_LETTERS) {
                value_next = place + 1 == letters.len();
            }
        }
    }
    false
}

/// The failure of an argument error. It holds no value from the command
/// line.
fn argument_failure() -> Failure {
    usage_failure(
        "the command line is not valid.",
        "Run jag --help to see the usage.",
    )
}

/// The document of a failure: the summary has what jag counted before it. It
/// fits in `limit` (STORY-26).
fn error_document(summary: &mut Summary, failure: Failure, limit: Option<usize>) -> Vec<u8> {
    let code = failure.exit_code;
    summary.error = Some(failure);
    let mut buffer = Vec::new();
    // A write to a `Vec` does not fail.
    let _ = write_document(&mut buffer, summary, &[], limit, &|_| code);
    buffer
}

/// Parse with `try_parse`, and never write the text of a clap error: clap
/// repeats a value from the command line in many errors (CV-6, decision 26).
fn parse() -> Result<Cli, ExitCode> {
    match parse_from(std::env::args_os()) {
        Ok(cli) => Ok(cli),
        Err(error) => match error.kind() {
            // These texts hold no value from the command line.
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                let _ = error.print();
                Err(ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2)))
            }
            kind => {
                // An unknown option names itself and never a value
                // (FEATURE-13, decision 18).
                let named = (kind == ErrorKind::UnknownArgument)
                    .then(|| grep::unknown_option_failure(&error))
                    .flatten();
                if json_requested(std::env::args_os()) {
                    let failure = named.unwrap_or_else(argument_failure);
                    // The document holds only fixed texts, so it fits.
                    let buffer =
                        error_document(&mut Summary::default(), failure, Some(DEFAULT_SIZE_LIMIT));
                    // A failed write goes to the error stream (decision 25).
                    let code = emit(&mut io::stdout(), &mut io::stderr(), &buffer, 2);
                    return Err(ExitCode::from(code));
                }
                match named {
                    Some(failure) => eprint!("{}", failure.render()),
                    None => eprintln!("{USAGE_ERROR}"),
                }
                Err(ExitCode::from(2))
            }
        },
    }
}

/// Parse a command line. The order of the globs comes from the matches, which
/// `Cli::try_parse_from` does not give (FEATURE-13, decision 27).
fn parse_from(args: impl IntoIterator<Item = OsString>) -> Result<Cli, clap::Error> {
    let matches = Cli::command().try_get_matches_from(args)?;
    let mut cli = Cli::from_arg_matches(&matches)?;
    cli.globs = grep::ordered_globs(&matches);
    Ok(cli)
}

/// Write a failure and give the exit code (CV-1, CV-3). In text mode the
/// failure goes to the error stream. In JSON mode it goes to standard output,
/// as one document (decision 21).
fn finish(
    result: Result<u8, Failure>,
    mode: Mode,
    summary: &mut Summary,
    limit: Option<usize>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code),
        Err(failure) if mode == Mode::Json => {
            let code = failure.exit_code;
            let buffer = error_document(summary, failure, limit);
            ExitCode::from(emit(stdout, stderr, &buffer, code))
        }
        Err(failure) => {
            let _ = write!(stderr, "{}", failure.render());
            ExitCode::from(failure.exit_code)
        }
    }
}

/// Write all output in one call. Gives `code` when the write works. A closed
/// pipe has two ends (decision 15). When the reader closes the pipe before
/// the search flow claims the output, the watch thread of `stop.rs` ends jag
/// by SIGPIPE, and this function does not run. When the reader closes it
/// during this write, jag writes no message and gives `code` (CV-4). Another
/// write error writes the failure as text on the error
/// stream, because standard output does not work, and gives exit 2
/// (decision 25).
fn emit(stdout: &mut dyn Write, stderr: &mut dyn Write, bytes: &[u8], code: u8) -> u8 {
    match stdout.write_all(bytes).and_then(|()| stdout.flush()) {
        Ok(()) => code,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => code,
        Err(_) => {
            let failure = write_failure();
            let _ = write!(stderr, "{}", failure.render());
            failure.exit_code
        }
    }
}

fn usage_failure(cause: &str, fix: &str) -> Failure {
    Failure {
        cause: cause.to_owned(),
        fix: fix.to_owned(),
        exit_code: 2,
    }
}

/// What the user typed for a search, after the checks of step 1.
#[derive(Debug)]
struct SearchArgs<'a> {
    /// A line matches one of the terms (FEATURE-13, `-e`).
    terms: Vec<&'a str>,
    /// `None` for a search with no question (FEATURE-12).
    question: Option<&'a str>,
    paths: Vec<PathBuf>,
    /// The options `-i`, `-w`, and `-F`.
    matching: MatchOptions,
    /// What plain grep writes: the lines, or `-l`, `-c`, or `-q`.
    report: Report,
    /// The lines around a match. `None`: no option gave them.
    context: Option<ContextLines>,
    /// The globs of `--include` and `--exclude`.
    files: FileFilter,
    /// The parsed `--timeout`, `--retry-wait`, `--base-url`, and `--model`.
    settings: SettingOptions,
    /// The parsed `--max-scored`.
    cost_limit: Option<usize>,
    /// The parsed `--max-chars` or `--no-max-chars`.
    size_limit: Option<SizeLimit>,
}

/// Step 1: the arguments of a search. No message shows a value.
fn search_args(cli: &Cli) -> Result<SearchArgs<'_>, Failure> {
    // With -e, each word is a path. With no -e, the first word is the term
    // (FEATURE-13, decision 10).
    let words = grep::words(cli);
    let (terms, path_words) = if cli.terms.is_empty() {
        let Some((term, rest)) = words.split_first() else {
            return Err(usage_failure(
                "a search needs a term.",
                "Use: jag [OPTIONS] TERM [PATH...]",
            ));
        };
        let term = term.to_str().ok_or_else(argument_failure)?;
        (vec![term], rest.to_vec())
    } else {
        (cli.terms.iter().map(String::as_str).collect(), words)
    };
    // A search needs no path. The paths `-` and "" are usage errors, in the
    // order of the command line (FEATURE-14, decision 8).
    for path in &path_words {
        if *path == OsStr::new("-") {
            return Err(usage_failure(
                "the path - is not valid.",
                "To search the text of a pipe, give no path. To search a file with the name -, give ./-.",
            ));
        }
        if path.is_empty() {
            return Err(usage_failure(
                "a path is empty.",
                "Give the path of a file or a folder, or give no path.",
            ));
        }
    }
    let question = cli.ask.as_deref();
    if let Some(question) = question {
        if question.trim().is_empty() {
            return Err(usage_failure(
                "the question is blank.",
                "Give a question that says what you look for.",
            ));
        }
        if question.len() > MAX_QUESTION_BYTES {
            return Err(usage_failure(
                "the question is longer than 12,000 bytes.",
                "Use a shorter question.",
            ));
        }
    }
    // Then the options of FEATURE-13 (decision 30).
    let report = grep::report(cli, question.is_some())?;
    let context = grep::context_lines(cli, question.is_some())?;
    let files = grep::file_filter(&cli.globs)?;
    // The time options come after the other checks and before the key
    // (decision 11). The timeout is checked first.
    let times = time_options(cli)?;
    let cost_limit = cli
        .cost_limit
        .as_deref()
        .map(|text| parse_cost_limit(text, SettingSource::Option))
        .transpose()
        .map_err(Failure::from)?;
    let size_limit = if cli.no_size_limit {
        Some(SizeLimit::Off)
    } else {
        cli.size_limit
            .as_deref()
            .map(|text| parse_size_limit(text, SettingSource::Option).map(SizeLimit::Chars))
            .transpose()
            .map_err(Failure::from)?
    };
    let settings = address_options(cli, times)?;
    Ok(SearchArgs {
        terms,
        question,
        paths: path_words.into_iter().map(PathBuf::from).collect(),
        matching: MatchOptions::of(cli),
        report,
        context,
        files,
        settings,
        cost_limit,
        size_limit,
    })
}

/// The checks of `--timeout` and `--retry-wait`, in this order.
fn time_options(cli: &Cli) -> Result<(Option<Duration>, Option<Duration>), Failure> {
    let timeout = parse_option(cli.timeout.as_deref(), TimeSetting::Timeout)?;
    let retry_wait = parse_option(cli.retry_wait.as_deref(), TimeSetting::RetryWait)?;
    Ok((timeout, retry_wait))
}

/// The checks of `--base-url` and `--model`, in this order, with the time
/// options that came before. A blank value, or an address that the address
/// rule refuses, gives exit 2. No message shows the value.
fn address_options(
    cli: &Cli,
    times: (Option<Duration>, Option<Duration>),
) -> Result<SettingOptions, Failure> {
    let base_url = cli
        .base_url
        .as_deref()
        .map(|text| {
            normalize_base_url(text).ok_or(ConfigError::BaseUrlBad {
                source: SettingSource::Option,
            })
        })
        .transpose()
        .map_err(Failure::from)?;
    let model = cli
        .model
        .as_deref()
        .map(|text| {
            let text = text.trim();
            if text.is_empty() {
                Err(ConfigError::ModelBad {
                    source: SettingSource::Option,
                })
            } else {
                Ok(text.to_owned())
            }
        })
        .transpose()
        .map_err(Failure::from)?;
    Ok(SettingOptions {
        base_url,
        model,
        times,
    })
}

fn parse_option(text: Option<&str>, setting: TimeSetting) -> Result<Option<Duration>, Failure> {
    text.map(|text| parse_seconds(text, setting, SettingSource::Option))
        .transpose()
        .map_err(Failure::from)
}

/// The search flow (spec "The search flow"). Gives the exit code: 0 for a
/// good answer, and 1 for no match or no good answer.
fn search(
    cli: &Cli,
    env: &Env,
    context: &Context,
    summary: &mut Summary,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    let Context {
        mode,
        stop,
        progress,
        stdout_is_terminal,
    } = *context;
    let args = search_args(cli)?;
    // The option is known now. The variable comes with the other settings.
    if let Some(option) = args.size_limit {
        stop.set_size_limit(option.chars());
    }
    // With no question, jag works as grep: no key, no model setting, and no
    // model call (FEATURE-12, decision 4).
    let Some(question) = args.question else {
        return plain(cli, &args, env, context, summary, stdout, stderr);
    };
    // The settings come before the key (decision 27). The config file is
    // read at most once.
    let file = ConfigFile::new(env, &RealFs);
    let settings = model_settings(env, &file, &args.settings).map_err(Failure::from)?;
    let (key, _source) =
        find_key(env, &file, settings.route, &settings.base_url_origin).map_err(Failure::from)?;
    let limit = cost_limit(env, args.cost_limit).map_err(Failure::from)?;
    let size = size_limit(env, args.size_limit)
        .map_err(Failure::from)?
        .chars();
    stop.set_size_limit(size);

    let matcher = build_matcher(&args.terms, args.matching)?;

    let options = ScanOptions {
        files: args.files.clone(),
        ..ScanOptions::default()
    };
    let (report, input) = scan_search(&args.paths, &matcher, &options);
    let windows = plan_windows(&report.files, args.context.unwrap_or(ContextLines::DEFAULT));
    let total = windows.len();
    // Only a search with more results than the limit pre-ranks them, from
    // the text of the scan, with no file read (decisions 8 and 15).
    let kept: Vec<_> = if total > limit {
        let texts = match_texts(&report.files, &windows);
        let best = prerank::best(&texts, question, limit);
        let mut windows: Vec<Option<_>> = windows.into_iter().map(Some).collect();
        best.into_iter()
            .filter_map(|index| windows[index].take())
            .collect()
    } else {
        windows
    };
    let (snippets, read_warnings) = read_snippets(&kept, input.as_deref());

    // A failure after this point finds in the summary what jag counted.
    record(
        summary,
        &report,
        &read_warnings,
        (total, kept.len(), total - kept.len()),
        context,
        stderr,
    );

    if snippets.is_empty() {
        return no_result(&report, context, summary, size, stdout, stderr);
    }
    let cost_limit_reached = total > limit;
    if cost_limit_reached && mode == Mode::Text {
        stop.notice(
            stderr,
            &format!(
                "jag: cost limit reached: scored {} of {total} results, skipped {}.",
                kept.len(),
                total - kept.len()
            ),
        );
    }

    let score_settings = ScoreSettings::new(settings);
    progress.start(snippets.len());
    let scores = {
        let _clear = ClearOnDrop(progress);
        score(
            &snippets,
            question,
            &key,
            &score_settings,
            stop.flag(),
            &|count| progress.advance(count),
        )
    };
    let scores = match scores {
        Ok(scores) => scores,
        // Only a stop sets the flag, and the stop writes the failure.
        Err(ScoreError::Stopped) => wait_for_the_end(),
        Err(error) => return Err(error.failure(&score_settings)),
    };
    let ranked = rank(snippets, scores);

    let no_good_answer = is_no_good_answer(&ranked);
    let exit_code = |left_out: bool| {
        search_exit_code(SearchOutcome {
            path_missing: report.path_missing(),
            no_good_answer,
            cost_limit_reached,
            left_out,
        })
    };

    // All output stays in memory until here, so a failure writes no partial
    // result (CV-2).
    let mut buffer = Vec::new();
    let code = match mode {
        Mode::Text => {
            let find = |line: &[u8]| find_ranges(&matcher, line);
            let paint = colored(cli, env, stdout_is_terminal).then_some(&find as Paint);
            let report =
                write_text(&mut buffer, &ranked, paint, size).map_err(|_| write_failure())?;
            // The size notice comes after the cost notice, and before the
            // output, so it shows also when the reader closes the pipe.
            if let Some(notice) = size_notice(&report.size, size) {
                stop.notice(stderr, &notice);
            }
            exit_code(report.size.limited())
        }
        Mode::Json => {
            summary.no_good_answer = no_good_answer;
            let results: Vec<&Snippet> = ranked.iter().map(|(snippet, _)| snippet).collect();
            let report = write_document(&mut buffer, summary, &results, size, &exit_code)
                .map_err(|_| write_failure())?;
            exit_code(report.limited())
        }
    };
    // A reader that closed the pipe gives no message and the code of the
    // search (CV-4).
    Ok(emit_claimed(stop, stdout, stderr, &buffer, code))
}

/// The flow with no question (FEATURE-12): the matching lines as grep gives
/// them, with no key, no model setting, and no model call.
fn plain(
    cli: &Cli,
    args: &SearchArgs,
    env: &Env,
    context: &Context,
    summary: &mut Summary,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    let Context {
        mode,
        stop,
        stdout_is_terminal,
        ..
    } = *context;
    // Plain grep reads no key, so a bad variable gives exit 2 at once.
    let size = size_limit(env, args.size_limit)
        .map_err(Failure::from)?
        .chars();
    stop.set_size_limit(size);
    let matcher = build_matcher(&args.terms, args.matching)?;
    let options = ScanOptions {
        files: args.files.clone(),
        keep_files_with_no_match: args.report == Report::Counts,
        ..ScanOptions::default()
    };
    let (report, input) = scan_search(&args.paths, &matcher, &options);
    if args.report != Report::Lines {
        let colored = colored(cli, env, stdout_is_terminal);
        return grep::write_report(
            args.report,
            colored,
            &report,
            context,
            summary,
            stdout,
            stderr,
        );
    }
    // With an option for the lines around a match, groups of lines. With none,
    // one line for each match (FEATURE-13, decision 15).
    let windows = match args.context {
        Some(lines) => plan_windows(&report.files, lines),
        None => line_windows(&report.files),
    };
    let (lines, read_warnings) = read_snippets(&windows, input.as_deref());
    record(
        summary,
        &report,
        &read_warnings,
        (windows.len(), 0, 0),
        context,
        stderr,
    );

    if lines.is_empty() {
        return no_result(&report, context, summary, size, stdout, stderr);
    }
    // A path that does not exist gives 2 also when lines match, as in grep
    // (FEATURE-14, decision 20).
    let exit_code = |left_out: bool| {
        search_exit_code(SearchOutcome {
            path_missing: report.path_missing(),
            no_good_answer: false,
            cost_limit_reached: false,
            left_out,
        })
    };
    let mut buffer = Vec::new();
    let code = match mode {
        Mode::Text => {
            let find = |line: &[u8]| find_ranges(&matcher, line);
            let paint = colored(cli, env, stdout_is_terminal).then_some(&find as Paint);
            let report = write_lines(&mut buffer, &lines, paint, args.context.is_some(), size)
                .map_err(|_| write_failure())?;
            if let Some(notice) = size_notice(&report, size) {
                stop.notice(stderr, &notice);
            }
            exit_code(report.limited())
        }
        Mode::Json => {
            let results: Vec<&Snippet> = lines.iter().collect();
            let report = write_document(&mut buffer, summary, &results, size, &exit_code)
                .map_err(|_| write_failure())?;
            exit_code(report.limited())
        }
    };
    Ok(emit_claimed(stop, stdout, stderr, &buffer, code))
}

/// The byte ranges of the matches in one line, for the color.
fn find_ranges(matcher: &RegexMatcher, line: &[u8]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    // A `RegexMatcher` gives no error (`NoError`).
    let _ = matcher.find_iter(line, |found| {
        ranges.push(found.start()..found.end());
        true
    });
    ranges
}

/// True when the text output gets color (decisions 5 and 6 of FEATURE-15).
fn colored(cli: &Cli, env: &Env, stdout_is_terminal: bool) -> bool {
    let term = (env.vars)("TERM");
    color_on(cli.color_when(), stdout_is_terminal, term.as_deref())
}

/// Put what the scan counted in the summary, and write the notices of text
/// mode. `counts` is the results found, scored, and skipped.
fn record(
    summary: &mut Summary,
    report: &ScanReport,
    read_warnings: &[WalkWarning],
    counts: (usize, usize, usize),
    context: &Context,
    stderr: &mut dyn Write,
) {
    let (found, scored, skipped) = counts;
    summary.files_skipped = report.skipped;
    summary.warnings = report
        .warnings
        .iter()
        .chain(read_warnings)
        .cloned()
        .collect();
    summary.results_found = found as u64;
    summary.results_scored = scored as u64;
    summary.results_skipped = skipped as u64;
    context.stop.count(summary);

    // CRITERION-85: in text mode, the notice and the warnings go to the error
    // stream. In JSON mode they are fields of the document.
    if context.mode == Mode::Text {
        if let Some(notice) = report.skipped.notice() {
            context.stop.notice(stderr, &notice);
        }
        for warning in &summary.warnings {
            context.stop.notice(stderr, &warning.message());
        }
    }
}

/// A search with no result: exit 2 when a path does not exist, else 1
/// (FEATURE-14, decision 20), and in structured mode the document.
fn no_result(
    report: &ScanReport,
    context: &Context,
    summary: &mut Summary,
    limit: Option<usize>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    let code = if report.path_missing() { 2 } else { 1 };
    if context.mode == Mode::Json {
        let mut buffer = Vec::new();
        write_document(&mut buffer, summary, &[], limit, &|_| code).map_err(|_| write_failure())?;
        return Ok(emit_claimed(context.stop, stdout, stderr, &buffer, code));
    }
    Ok(code)
}

/// The scan of a search (FEATURE-14, "The search flow"). With paths, jag
/// never reads standard input (decision 4). With no path, it searches the
/// piped text or the current folder. Gives the report and the piped text.
fn scan_search(
    paths: &[PathBuf],
    matcher: &RegexMatcher,
    options: &ScanOptions,
) -> (ScanReport, Option<Vec<u8>>) {
    if !paths.is_empty() {
        return (scan(paths, matcher, options), None);
    }
    match no_path_input() {
        Input::PipedText(bytes) => {
            let report = scan_input(&bytes, matcher, options.keep_files_with_no_match);
            (report, Some(bytes))
        }
        Input::CurrentFolder => (scan_current_folder(matcher, options), None),
        Input::ReadError(warning) => {
            let mut report = ScanReport::empty();
            report.warnings.push(warning);
            (report, None)
        }
    }
}

/// `emit` for the search flow: it takes the output first. When a stop owns
/// the output, the search flow waits for the end of the process.
fn emit_claimed(
    stop: &Stop,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    bytes: &[u8],
    code: u8,
) -> u8 {
    claim_or_wait(stop);
    emit(stdout, stderr, bytes, code)
}

/// The search flow takes the output. When a stop owns it, the search flow
/// waits for the end of the process.
fn claim_or_wait(stop: &Stop) {
    if !stop.claim_for_search() {
        wait_for_the_end();
    }
}

/// The readiness check (FEATURE-02). Gives the exit code of the first item
/// that fails, or 0. A usage error stops the check before the first item
/// (decision 17).
fn check(
    cli: &Cli,
    env: &Env,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    // The timeout is checked first.
    let options = address_options(cli, time_options(cli)?)?;
    let file = ConfigFile::new(env, &RealFs);
    // A file error shows on the items. Each other error stops the check.
    let settings = match model_settings(env, &file, &options) {
        Ok(settings) => Ok(ScoreSettings::new(settings)),
        Err(error) if error.is_file_error() => Err(error),
        Err(error) => return Err(Failure::from(error)),
    };
    let report = check::run(env, &file, settings, &|key, settings| probe(key, settings));

    // The lines first, then the failures. A reader that closes the pipe gives
    // no message and keeps the exit code (CV-4).
    let mut lines = report.lines().into_bytes();
    match stdout.write_all(&lines).and_then(|()| stdout.flush()) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
        Err(_) => return Err(write_failure()),
    }
    lines.clear();
    for failure in report.failures() {
        lines.extend_from_slice(failure.render().as_bytes());
    }
    match stderr.write_all(&lines).and_then(|()| stderr.flush()) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
        Err(_) => return Err(write_failure()),
    }
    Ok(report.exit_code())
}

/// What a search that got its scores found.
struct SearchOutcome {
    /// A path of the command line does not exist (FEATURE-14, decision 7).
    path_missing: bool,
    no_good_answer: bool,
    cost_limit_reached: bool,
    /// The size limit left out a result or cut the top result.
    left_out: bool,
}

/// IF-4 order with FEATURE-14 decision 20: 2 wins, then 1, then 7, then 8,
/// then 0 (DECISION-13, DECISION-47).
fn search_exit_code(outcome: SearchOutcome) -> u8 {
    if outcome.path_missing {
        2
    } else if outcome.no_good_answer {
        1
    } else if outcome.cost_limit_reached {
        7
    } else if outcome.left_out {
        8
    } else {
        0
    }
}

fn write_failure() -> Failure {
    usage_failure(
        "jag could not write the results.",
        "Check where the output goes.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn json_requested_reads_the_option_before_the_first_double_dash() {
        assert!(json_requested(args(&["jag", "--json"])));
        assert!(json_requested(args(&["jag", "x", "--json"])));
        assert!(json_requested(args(&["jag", "--json", "--", "--json"])));
        assert!(!json_requested(args(&["jag", "--", "--json"])));
        assert!(!json_requested(args(&["jag", "--json=1"])));
        assert!(!json_requested(args(&["jag"])));
        assert!(!json_requested(args(&[])));
    }

    // FEATURE-13, decision 38: a value of an option is not an option.
    #[test]
    fn json_requested_does_not_read_the_value_of_an_option() {
        for list in [
            &["jag", "-e", "--json", "--nope"][..],
            &["jag", "--regexp", "--json"],
            &["jag", "-ie", "--json"],
            &["jag", "-A", "--json"],
            &["jag", "-B", "--json"],
            &["jag", "-C", "--json"],
            &["jag", "--after-context", "--json"],
            &["jag", "-e--json"],
            &["jag", "--regexp=--json"],
        ] {
            assert!(!json_requested(args(list)), "{list:?}");
        }
        // `-ei`: `-e` takes `i`, so the next word `--json` is the option.
        assert!(json_requested(args(&["jag", "-ei", "--json"])));
    }

    #[test]
    fn json_requested_reads_a_double_dash_that_is_a_value_as_a_value() {
        assert!(json_requested(args(&[
            "jag", "-e", "--", "--json", "--nope"
        ])));
        assert!(json_requested(args(&["jag", "--regexp", "--", "--json"])));
        assert!(!json_requested(args(&["jag", "-e", "--", "--", "--json"])));
        assert!(!json_requested(args(&["jag", "-e", "x", "--", "--json"])));
    }

    #[test]
    fn json_requested_reads_json_after_the_other_options_with_a_value() {
        assert!(json_requested(args(&["jag", "--ask", "--json", "t"])));
        assert!(json_requested(args(&[
            "jag",
            "--include",
            "*.rs",
            "--json"
        ])));
        assert!(json_requested(args(&["jag", "-e", "a", "--json"])));
        assert!(json_requested(args(&["jag", "--include", "--json"])));
        assert!(json_requested(args(&["jag", "--nope", "--json"])));
        assert!(!json_requested(args(&["jag", "--json=1"])));
    }

    // Risk 12: the lists of `json_requested` stay the same as `Cli`.
    #[test]
    fn the_hyphen_value_lists_agree_with_cli() {
        let command = Cli::command();
        let hyphen: Vec<_> = command
            .get_arguments()
            .filter(|arg| arg.is_allow_hyphen_values_set())
            .collect();
        let mut longs: Vec<String> = hyphen
            .iter()
            .filter_map(|arg| arg.get_long().map(|long| format!("--{long}")))
            .collect();
        let mut letters: Vec<char> = hyphen.iter().filter_map(|arg| arg.get_short()).collect();
        longs.sort();
        letters.sort();
        let mut want_longs = HYPHEN_VALUE_OPTIONS.map(String::from).to_vec();
        let mut want_letters = HYPHEN_VALUE_LETTERS.to_vec();
        want_longs.sort();
        want_letters.sort();
        assert_eq!(longs, want_longs);
        assert_eq!(letters, want_letters);
    }

    #[test]
    fn color_on_follows_grep_color_auto() {
        let term = |text: &'static str| Some(OsStr::new(text));
        assert!(color_on(ColorWhen::Always, false, None));
        assert!(color_on(ColorWhen::Auto, true, term("xterm")));
        assert!(color_on(ColorWhen::Auto, true, term("")));
        assert!(!color_on(ColorWhen::Never, true, term("xterm")));
        assert!(!color_on(ColorWhen::Auto, true, term("dumb")));
        assert!(!color_on(ColorWhen::Auto, true, None));
        assert!(!color_on(ColorWhen::Auto, false, term("xterm")));
    }

    type Parts = (Option<String>, Option<String>, Vec<String>);

    /// The question, the term and the paths of a command line.
    fn parts(list: &[&str]) -> Result<Parts, clap::Error> {
        let mut all = vec!["jag"];
        all.extend_from_slice(list);
        Cli::try_parse_from(all).map(|cli| {
            let paths = cli
                .paths
                .iter()
                .map(|path| path.display().to_string())
                .collect();
            let term = cli.term.map(|term| term.to_string_lossy().into_owned());
            (cli.ask, term, paths)
        })
    }

    fn some(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    #[test]
    fn the_option_ask_gives_the_question_in_each_form() {
        for list in [
            &["--ask", "q", "t", "p"][..],
            &["--ask=q", "t", "p"],
            &["t", "--ask", "q", "p"],
            &["t", "p", "--ask=q"],
        ] {
            let (ask, term, paths) = parts(list).expect("parse");
            assert_eq!((ask, term), (some("q"), some("t")), "{list:?}");
            assert_eq!(paths, ["p"], "{list:?}");
        }
        let (ask, term, paths) = parts(&["t", "p"]).expect("parse");
        assert_eq!((ask, term, paths), (None, some("t"), vec!["p".to_string()]));
    }

    #[test]
    fn a_question_that_starts_with_a_dash_needs_an_equals_sign() {
        let (ask, ..) = parts(&["--ask=-x", "t", "p"]).expect("parse");
        assert_eq!(ask, some("-x"));
        assert!(parts(&["--ask", "-x", "t", "p"]).is_err());
        assert!(parts(&["--ask", "--json", "t", "p"]).is_err());
    }

    #[test]
    fn the_option_ask_is_refused_when_it_is_wrong() {
        assert!(parts(&["--ask"]).is_err());
        assert!(parts(&["--ask", "a", "--ask", "b", "t", "p"]).is_err());
        assert!(parts(&["--check", "--ask", "q"]).is_err());
        assert!(parts(&["--setup", "--ask", "q"]).is_err());
        assert!(parts(&["--forget-key", "--ask", "q"]).is_err());
    }

    #[test]
    fn after_a_double_dash_the_word_ask_is_a_term() {
        let (ask, term, paths) = parts(&["--", "--ask", "dir"]).expect("parse");
        assert_eq!((ask, term), (None, some("--ask")));
        assert_eq!(paths, ["dir"]);
    }

    fn color_of(list: &[&str]) -> Result<ColorWhen, clap::Error> {
        let mut all = vec!["jag"];
        all.extend_from_slice(list);
        all.extend_from_slice(&["term", "question", "path"]);
        Cli::try_parse_from(all).map(|cli| cli.color_when())
    }

    #[test]
    fn the_option_color_takes_never_always_and_auto() {
        assert_eq!(color_of(&[]).expect("parse"), ColorWhen::Auto);
        assert_eq!(color_of(&["--color"]).expect("parse"), ColorWhen::Auto);
        assert_eq!(color_of(&["--color=auto"]).expect("parse"), ColorWhen::Auto);
        assert_eq!(
            color_of(&["--color=always", "--color=never"]).expect("parse"),
            ColorWhen::Never
        );
        assert_eq!(
            color_of(&["--colour=always"]).expect("parse"),
            ColorWhen::Always
        );
        assert!(color_of(&["--color=sometimes"]).is_err());
    }

    #[test]
    fn a_value_of_color_needs_an_equals_sign() {
        // `never` is the term here, as in grep.
        let cli =
            Cli::try_parse_from(["jag", "--color", "never", "question", "path"]).expect("parse");
        assert_eq!(cli.color_when(), ColorWhen::Auto);
        assert_eq!(cli.term.as_deref(), Some(OsStr::new("never")));
    }

    fn scoring_step_that_fails(progress: &Progress) -> Result<(), Failure> {
        let _clear = ClearOnDrop(progress);
        Err(usage_failure("a cause.", "a fix."))
    }

    #[test]
    fn clear_on_drop_removes_the_line_before_the_failure_text() {
        use crate::output::progress::tests::Shared;
        let line = Shared::default();
        let progress = Progress::new(Box::new(line.clone()), true);
        progress.start(70);
        assert!(scoring_step_that_fails(&progress).is_err());
        assert!(line.text().ends_with(&format!("\r{}\r", " ".repeat(37))));
    }

    fn code(no_good_answer: bool, cost_limit_reached: bool, left_out: bool) -> u8 {
        code_with(false, no_good_answer, cost_limit_reached, left_out)
    }

    fn code_with(
        path_missing: bool,
        no_good_answer: bool,
        cost_limit_reached: bool,
        left_out: bool,
    ) -> u8 {
        search_exit_code(SearchOutcome {
            path_missing,
            no_good_answer,
            cost_limit_reached,
            left_out,
        })
    }

    #[test]
    fn the_exit_code_follows_the_order_2_1_7_8_0() {
        for no_good_answer in [false, true] {
            for cost_limit_reached in [false, true] {
                for left_out in [false, true] {
                    assert_eq!(
                        code_with(true, no_good_answer, cost_limit_reached, left_out),
                        2
                    );
                    if no_good_answer {
                        assert_eq!(code(true, cost_limit_reached, left_out), 1);
                    }
                }
            }
        }
        assert_eq!(code(false, true, true), 7);
        assert_eq!(code(false, true, false), 7);
        assert_eq!(code(false, false, true), 8);
        assert_eq!(code(false, false, false), 0);
    }

    fn search_failure(list: &[&str]) -> Option<String> {
        let mut all = vec!["jag"];
        all.extend_from_slice(list);
        let cli = Cli::try_parse_from(all).expect("parse");
        search_args(&cli).err().map(|failure| failure.render())
    }

    // Decision 1 of FEATURE-14: a search needs no path.
    #[test]
    fn a_search_with_no_path_passes_step_1() {
        assert_eq!(search_failure(&["needle"]), None);
        assert_eq!(search_failure(&["--ask", "q", "needle"]), None);
        let failure = search_failure(&["--json"]).expect("no term");
        assert!(failure.contains("Use: jag [OPTIONS] TERM [PATH...]"));
    }

    // Decision 8 of FEATURE-14.
    #[test]
    fn the_paths_dash_and_empty_are_usage_errors_in_the_order_of_the_command_line() {
        let dash = search_failure(&["needle", "a.txt", "-"]).expect("dash");
        assert!(dash.contains("the path - is not valid."));
        assert!(dash.contains("give ./-."));
        let empty = search_failure(&["needle", "a.txt", ""]).expect("empty");
        assert!(empty.contains("a path is empty."));
        assert!(!empty.contains("  "));
        let first = search_failure(&["needle", "", "-"]).expect("both");
        assert!(first.contains("a path is empty."));
        assert_eq!(search_failure(&["needle", "./-"]), None);
    }
}
