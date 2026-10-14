//! C-cli: option parsing, dispatch, the search flow, and the exit code.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use clap::error::ErrorKind;
use grep_regex::RegexMatcherBuilder;

use crate::config::{
    self, Env, RealFs, SettingSource, TimeSetting, TtyPrompter, cost_limit, find_key,
    model_settings, parse_cost_limit, parse_seconds,
};
use crate::failure::Failure;
use crate::output::write_text;
use crate::prerank;
use crate::scorer::{ScoreSettings, rank, score};
use crate::search::snippet::{match_texts, plan_windows, read_snippets};
use crate::search::{ScanOptions, scan};

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
    override_usage = "jag [--timeout SECONDS] [--retry-wait SECONDS] [--max-scored COUNT] TERM \"QUESTION\" [PATH...]\n       jag --setup\n       jag --forget-key"
)]
struct Cli {
    /// Store your TypeSafe API key, with no echo
    #[arg(long, conflicts_with_all = ["forget_key", "term", "question", "paths"])]
    setup: bool,

    /// Remove the stored TypeSafe API key
    #[arg(long = "forget-key", conflicts_with_all = ["term", "question", "paths"])]
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

    /// The regular expression that the lines must match
    #[arg(value_name = "TERM")]
    term: Option<String>,

    /// What you look for: the model ranks the matches by this question
    #[arg(value_name = "QUESTION")]
    question: Option<String>,

    /// The files and folders to search
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
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
    } else {
        search(&cli, &env, &mut stdout, &mut stderr)
    };
    finish(result)
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
            _ => {
                eprintln!("{USAGE_ERROR}");
                Err(ExitCode::from(2))
            }
        },
    }
}

/// Write a failure to the error stream and give the exit code (CV-1, CV-3).
fn finish(result: Result<u8, Failure>) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code),
        Err(failure) => {
            eprint!("{}", failure.render());
            ExitCode::from(failure.exit_code)
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
    question: &'a str,
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
            "a search needs a term and a question.",
            "Use: jag TERM \"QUESTION\" PATH...",
        ));
    };
    let Some(question) = cli.question.as_deref() else {
        return Err(usage_failure(
            "a search needs a question.",
            "Use: jag TERM \"QUESTION\" PATH...",
        ));
    };
    if cli.paths.is_empty() {
        return Err(usage_failure(
            "a search needs a path.",
            "Searching standard input comes later.",
        ));
    }
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
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<u8, Failure> {
    let args = search_args(cli)?;
    // The key comes before any file read (decision 18).
    let (key, _source) = find_key(env, &RealFs).map_err(Failure::from)?;
    let settings = model_settings(env, args.times).map_err(Failure::from)?;
    let limit = cost_limit(env, args.cost_limit).map_err(Failure::from)?;

    let matcher = RegexMatcherBuilder::new()
        .line_terminator(Some(b'\n'))
        .build(args.term)
        .map_err(|_| {
            usage_failure(
                "the term is not a valid regular expression.",
                "Check the term. It uses the regular expression rules of ripgrep.",
            )
        })?;

    let report = scan(args.paths, &matcher, &ScanOptions::default());
    let windows = plan_windows(&report.files);
    let total = windows.len();
    // Only a search with more results than the limit pre-ranks them, from
    // the text of the scan, with no file read (decisions 8 and 15).
    let kept: Vec<_> = if total > limit {
        let texts = match_texts(&report.files, &windows);
        let best = prerank::best(&texts, args.question, limit);
        let mut windows: Vec<Option<_>> = windows.into_iter().map(Some).collect();
        best.into_iter()
            .filter_map(|index| windows[index].take())
            .collect()
    } else {
        windows
    };
    let (snippets, read_warnings) = read_snippets(&kept);

    // CRITERION-85: the notice and the warnings go to the error stream.
    if let Some(notice) = report.skipped.notice() {
        let _ = writeln!(stderr, "{notice}");
    }
    for warning in report.warnings.iter().chain(&read_warnings) {
        let _ = writeln!(stderr, "{}", warning.message());
    }

    if snippets.is_empty() {
        return Ok(1);
    }
    let cost_limit_reached = total > limit;
    if cost_limit_reached {
        let _ = writeln!(
            stderr,
            "jag: cost limit reached: scored {} of {total} results, skipped {}.",
            kept.len(),
            total - kept.len()
        );
    }

    let score_settings = ScoreSettings::new(settings.base_url, settings.model, settings.times);
    let scores = score(&snippets, args.question, &key, &score_settings).map_err(Failure::from)?;
    let ranked = rank(snippets, scores);

    // All output stays in memory until here, so a failure writes no partial
    // result (CV-2).
    let mut buffer = Vec::new();
    let no_good_answer = write_text(&mut buffer, &ranked).map_err(|_| write_failure())?;
    match stdout.write_all(&buffer).and_then(|()| stdout.flush()) {
        Ok(()) => {}
        // The reader closed the pipe: end with the code of the search, with
        // no message (CV-4).
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {}
        Err(_) => return Err(write_failure()),
    }
    Ok(search_exit_code(SearchOutcome {
        no_good_answer,
        cost_limit_reached,
        left_out: false,
    }))
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
