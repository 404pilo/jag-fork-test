//! C-cli: option parsing, dispatch, the search flow, and the exit code.

mod stop;

use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use clap::error::ErrorKind;
use grep_matcher::Matcher;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};

use crate::check;
use crate::config::{
    self, Env, RealFs, SettingSource, TimeSetting, TtyPrompter, cost_limit, find_key,
    model_settings, parse_cost_limit, parse_seconds,
};
use crate::failure::Failure;
use crate::output::progress::Progress;
use crate::output::{Paint, Summary, is_no_good_answer, write_json, write_lines, write_text};
use crate::prerank;
use crate::scorer::{ScoreError, ScoreSettings, probe, rank, score};
use crate::search::snippet::{Snippet, line_windows, match_texts, plan_windows, read_snippets};
use crate::search::{ScanOptions, ScanReport, WalkWarning, scan};
use stop::Stop;

/// The longest question, in UTF-8 bytes (decision 30).
const MAX_QUESTION_BYTES: usize = 12_000;

/// The one line for each argument error. It never holds a value from the
/// command line, because a value can be a key (CV-6, decision 26).
const USAGE_ERROR: &str = "jag: the command line is not valid. Run jag --help to see the usage.";

/// Grep with a model that ranks the matches by how well they answer your question.
#[derive(Parser, Debug)]
#[command(
    name = "jag",
    version,
    arg_required_else_help = true,
    override_usage = "jag [OPTIONS] TERM [PATH...]\n       jag [OPTIONS] --ask QUESTION TERM [PATH...]\n       jag --check [--timeout SECONDS] [--retry-wait SECONDS]\n       jag --setup\n       jag --forget-key"
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

    /// Check that jag is ready: the key, the service, and the key file
    #[arg(long, conflicts_with_all = ["setup", "forget_key", "cost_limit", "term", "ask", "paths"])]
    check: bool,

    /// Store your TypeSafe API key, with no echo
    #[arg(long, conflicts_with_all = ["forget_key", "term", "ask", "paths"])]
    setup: bool,

    /// Remove the stored TypeSafe API key
    #[arg(long = "forget-key", conflicts_with_all = ["term", "ask", "paths"])]
    forget_key: bool,

    /// Seconds to wait for one model call, from 1 to 60 (default 4)
    #[arg(long, value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
    timeout: Option<String>,

    /// Seconds to wait before the one retry, from 1 to 60 (default 1)
    #[arg(long = "retry-wait", value_name = "SECONDS", conflicts_with_all = ["setup", "forget_key"])]
    retry_wait: Option<String>,

    /// The most results that the model scores, from 1 to 2,000 (default 200)
    #[arg(long = "max-scored", value_name = "COUNT", conflicts_with_all = ["setup", "forget_key"])]
    cost_limit: Option<String>,

    /// What you look for: the model ranks the matches by this question. With no --ask, jag works as grep: no model call, no cost, and no key
    #[arg(long, value_name = "QUESTION")]
    ask: Option<String>,

    /// The regular expression that the lines must match
    #[arg(value_name = "TERM")]
    term: Option<String>,

    /// The files and folders to search
    #[arg(value_name = "PATH")]
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
        result
    };
    finish(result, mode, &mut summary, &mut stdout, &mut stderr)
}

/// The form of the output (DECISION-27). Only `--json` gives `Json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Text,
    Json,
}

/// True when the command line holds `--json` before the first `--`. After
/// `--`, `--json` is a term or a path, not the option.
fn json_requested(args: impl IntoIterator<Item = OsString>) -> bool {
    args.into_iter()
        .skip(1)
        .take_while(|arg| arg != "--")
        .any(|arg| arg == "--json")
}

/// The failure of an argument error. It holds no value from the command
/// line.
fn argument_failure() -> Failure {
    usage_failure(
        "the command line is not valid.",
        "Run jag --help to see the usage.",
    )
}

/// The document of a failure: the summary has what jag counted before it.
fn error_document(summary: &mut Summary, failure: Failure) -> Vec<u8> {
    summary.exit_code = failure.exit_code;
    summary.error = Some(failure);
    let mut buffer = Vec::new();
    // A write to a `Vec` does not fail.
    let _ = write_json(&mut buffer, summary, &[]);
    buffer
}

/// Parse with `try_parse`, and never write the text of a clap error: clap
/// repeats a value from the command line in many errors (CV-6, decision 26).
fn parse() -> Result<Cli, ExitCode> {
    match Cli::try_parse() {
        Ok(cli) => Ok(cli),
        Err(error) => match error.kind() {
            // These texts hold no value from the command line.
            ErrorKind::DisplayHelp
            | ErrorKind::DisplayVersion
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                let _ = error.print();
                Err(ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2)))
            }
            _ if json_requested(std::env::args_os()) => {
                let buffer = error_document(&mut Summary::default(), argument_failure());
                // A failed write goes to the error stream (decision 25).
                let code = emit(&mut io::stdout(), &mut io::stderr(), &buffer, 2);
                Err(ExitCode::from(code))
            }
            _ => {
                eprintln!("{USAGE_ERROR}");
                Err(ExitCode::from(2))
            }
        },
    }
}

/// Write a failure and give the exit code (CV-1, CV-3). In text mode the
/// failure goes to the error stream. In JSON mode it goes to standard output,
/// as one document (decision 21).
fn finish(
    result: Result<u8, Failure>,
    mode: Mode,
    summary: &mut Summary,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code),
        Err(failure) if mode == Mode::Json => {
            let code = failure.exit_code;
            let buffer = error_document(summary, failure);
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
struct SearchArgs<'a> {
    term: &'a str,
    /// `None` for a search with no question (FEATURE-12).
    question: Option<&'a str>,
    paths: &'a [PathBuf],
    /// The parsed `--timeout` and `--retry-wait`.
    times: (Option<Duration>, Option<Duration>),
    /// The parsed `--max-scored`.
    cost_limit: Option<usize>,
}

/// Step 1: the arguments of a search. No message shows a value.
fn search_args(cli: &Cli) -> Result<SearchArgs<'_>, Failure> {
    let Some(term) = cli.term.as_deref() else {
        return Err(usage_failure(
            "a search needs a term.",
            "Use: jag [OPTIONS] TERM PATH...",
        ));
    };
    if cli.paths.is_empty() {
        return Err(usage_failure(
            "a search needs a path.",
            "Searching standard input comes later.",
        ));
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
    // The time options come after the other checks and before the key
    // (decision 11). The timeout is checked first.
    let timeout = parse_option(cli.timeout.as_deref(), TimeSetting::Timeout)?;
    let retry_wait = parse_option(cli.retry_wait.as_deref(), TimeSetting::RetryWait)?;
    let cost_limit = cli
        .cost_limit
        .as_deref()
        .map(|text| parse_cost_limit(text, SettingSource::Option))
        .transpose()
        .map_err(Failure::from)?;
    Ok(SearchArgs {
        term,
        question,
        paths: &cli.paths,
        times: (timeout, retry_wait),
        cost_limit,
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
    // With no question, jag works as grep: no key, no model setting, and no
    // model call (FEATURE-12, decision 4).
    let Some(question) = args.question else {
        return plain(cli, &args, env, context, summary, stdout, stderr);
    };
    // The key comes before any file read (decision 18).
    let (key, _source) = find_key(env, &RealFs).map_err(Failure::from)?;
    let settings = model_settings(env, args.times).map_err(Failure::from)?;
    let limit = cost_limit(env, args.cost_limit).map_err(Failure::from)?;

    let matcher = build_matcher(args.term)?;

    let report = scan(args.paths, &matcher, &ScanOptions::default());
    let windows = plan_windows(&report.files);
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
    let (snippets, read_warnings) = read_snippets(&kept);

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
        return no_result(context, summary, stdout, stderr);
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

    let score_settings = ScoreSettings::new(settings.base_url, settings.model, settings.times);
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
        Err(error) => return Err(Failure::from(error)),
    };
    let ranked = rank(snippets, scores);

    let no_good_answer = is_no_good_answer(&ranked);
    let code = search_exit_code(SearchOutcome {
        no_good_answer,
        cost_limit_reached,
        left_out: false,
    });

    // All output stays in memory until here, so a failure writes no partial
    // result (CV-2).
    let mut buffer = Vec::new();
    match mode {
        Mode::Text => {
            let find = |line: &[u8]| find_ranges(&matcher, line);
            let paint = colored(cli, env, stdout_is_terminal).then_some(&find as Paint);
            write_text(&mut buffer, &ranked, paint).map_err(|_| write_failure())?;
        }
        Mode::Json => {
            summary.no_good_answer = no_good_answer;
            summary.exit_code = code;
            let results: Vec<&Snippet> = ranked.iter().map(|(snippet, _)| snippet).collect();
            write_json(&mut buffer, summary, &results).map_err(|_| write_failure())?;
        }
    }
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
    let matcher = build_matcher(args.term)?;
    let report = scan(args.paths, &matcher, &ScanOptions::default());
    let windows = line_windows(&report.files);
    let (lines, read_warnings) = read_snippets(&windows);
    record(
        summary,
        &report,
        &read_warnings,
        (windows.len(), 0, 0),
        context,
        stderr,
    );

    if lines.is_empty() {
        return no_result(context, summary, stdout, stderr);
    }
    let mut buffer = Vec::new();
    match mode {
        Mode::Text => {
            let find = |line: &[u8]| find_ranges(&matcher, line);
            let paint = colored(cli, env, stdout_is_terminal).then_some(&find as Paint);
            write_lines(&mut buffer, &lines, paint).map_err(|_| write_failure())?;
        }
        Mode::Json => {
            summary.exit_code = 0;
            let results: Vec<&Snippet> = lines.iter().collect();
            write_json(&mut buffer, summary, &results).map_err(|_| write_failure())?;
        }
    }
    Ok(emit_claimed(stop, stdout, stderr, &buffer, 0))
}

fn build_matcher(term: &str) -> Result<RegexMatcher, Failure> {
    RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .build(term)
        .map_err(|_| {
            usage_failure(
                "the term is not a valid regular expression.",
                "Check the term. It uses the regular expression rules of ripgrep.",
            )
        })
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

/// A search with no result: exit 1, and in structured mode the document.
fn no_result(
    context: &Context,
    summary: &mut Summary,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    if context.mode == Mode::Json {
        summary.exit_code = 1;
        let mut buffer = Vec::new();
        write_json(&mut buffer, summary, &[]).map_err(|_| write_failure())?;
        return Ok(emit_claimed(context.stop, stdout, stderr, &buffer, 1));
    }
    Ok(1)
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
    let timeout = parse_option(cli.timeout.as_deref(), TimeSetting::Timeout)?;
    let retry_wait = parse_option(cli.retry_wait.as_deref(), TimeSetting::RetryWait)?;
    let settings = model_settings(env, (timeout, retry_wait)).map_err(Failure::from)?;
    let settings = ScoreSettings::new(settings.base_url, settings.model, settings.times);
    let report = check::run(env, &RealFs, &|key| probe(key, &settings));

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
    no_good_answer: bool,
    cost_limit_reached: bool,
    /// FEATURE-09 sets this when the size limit leaves out results.
    left_out: bool,
}

/// IF-4 order: 1 wins, then 7, then 8, then 0 (DECISION-13, DECISION-47).
fn search_exit_code(outcome: SearchOutcome) -> u8 {
    if outcome.no_good_answer {
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
            (cli.ask, cli.term, paths)
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
        assert_eq!(cli.term.as_deref(), Some("never"));
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
        search_exit_code(SearchOutcome {
            no_good_answer,
            cost_limit_reached,
            left_out,
        })
    }

    #[test]
    fn the_exit_code_follows_the_order_1_7_8_0() {
        for cost_limit_reached in [false, true] {
            for left_out in [false, true] {
                assert_eq!(code(true, cost_limit_reached, left_out), 1);
            }
        }
        assert_eq!(code(false, true, true), 7);
        assert_eq!(code(false, true, false), 7);
        assert_eq!(code(false, false, true), 8);
        assert_eq!(code(false, false, false), 0);
    }
}
