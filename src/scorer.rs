//! C-scorer: the model calls that score each result (STORY-13, STORY-15).
//!
//! `score` sends the results to the model service in calls of 32 results or
//! less, 6 calls in parallel, and gives one score for each result.

mod client;
pub mod cost;
pub mod prompt;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::config::{ApiKey, KEYS_PAGE, ModelSettings, Origin, Route};
use crate::failure::Failure;
use crate::search::snippet::{Snippet, SnippetLine, Source};
use client::{Attempt, Client};
use cost::Usage;

/// The version of the scoring prompt (ADR-0004).
pub const PROMPT_VERSION: &str = "jag-p1";
/// A top score below this is "no good answer" (ADR-0003).
pub const THRESHOLD: f64 = 0.4;
/// The calls that run at the same time.
const MAX_THREADS: usize = 6;
/// The page where the user adds credit (the docs give no page for it).
const CONSOLE_PAGE: &str = "https://console.typesafe.ai";

/// The address, the model, and the times of CV-5.
#[derive(Clone, Debug)]
pub struct ScoreSettings {
    pub base_url: String,
    pub model: String,
    /// Where the calls go.
    pub route: Route,
    /// The source of the base URL, for the fixes.
    pub base_url_origin: Origin,
    /// The source of the model, for the fixes.
    pub model_origin: Origin,
    /// The time of one attempt.
    pub timeout: Duration,
    /// The wait before the one retry.
    pub retry_wait: Duration,
    /// The time for all calls of a search. No call and no retry starts after
    /// it.
    pub deadline: Duration,
}

impl ScoreSettings {
    /// The times of CV-5. The deadline is `2 x timeout + retry_wait`: 9
    /// seconds with the defaults (4 s and 1 s), 180 seconds with the largest
    /// values (decision 5 of FEATURE-03).
    pub fn new(settings: ModelSettings) -> Self {
        let times = settings.times;
        Self {
            base_url: settings.base_url,
            model: settings.model,
            route: settings.route,
            base_url_origin: settings.base_url_origin,
            model_origin: settings.model_origin,
            timeout: times.timeout,
            retry_wait: times.retry_wait,
            deadline: 2 * times.timeout + times.retry_wait,
        }
    }
}

/// Why a search could not get its scores. No text here holds a value from
/// the command line, a body, an address, or the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScoreError {
    /// HTTP 401 or 403. Exit 4.
    KeyNotAccepted { status: u16 },
    /// HTTP 402. Exit 5.
    NoCredit,
    /// A timeout, or the deadline. Exit 6.
    TimedOut,
    /// No answer from the service for another reason. Exit 6.
    CannotConnect,
    /// HTTP 429. Exit 6.
    RateLimited,
    /// HTTP 500 to 599. Exit 6.
    ServerFailed { status: u16 },
    /// HTTP 300 to 399. Exit 6.
    Redirect,
    /// Any other status. Exit 6.
    UnexpectedStatus { status: u16 },
    /// HTTP 200 with a body that jag cannot read. Exit 6.
    BadResponse,
    /// A Ctrl-C or a closed pipe set the stop flag. Exit 130.
    Stopped,
}

impl ScoreError {
    /// One cause text and one fix text for each cause, for the route and the
    /// sources of the settings (FEATURE-03, decision 12; FEATURE-21, decision
    /// 17). No text holds the key, an address, a model name, or a body.
    pub fn failure(self, settings: &ScoreSettings) -> Failure {
        let (cause, fix, exit_code) = match settings.route {
            Route::Direct => self.direct_text(settings),
            Route::Gateway => self.gateway_text(settings),
        };
        Failure {
            cause,
            fix,
            exit_code,
        }
    }

    /// The texts of the direct route.
    fn direct_text(self, settings: &ScoreSettings) -> (String, String, u8) {
        let base = settings.base_url_origin.base_url_name();
        let model = settings.model_origin.model_name();
        match self {
            ScoreError::KeyNotAccepted { status } => (
                format!("the TypeSafe API did not accept the key (HTTP {status})."),
                format!("Check the key at {KEYS_PAGE}, then run jag --setup."),
                4,
            ),
            ScoreError::NoCredit => (
                "the TypeSafe account has no credit (HTTP 402).".to_owned(),
                format!("Add credit at {CONSOLE_PAGE}, then try again."),
                5,
            ),
            ScoreError::TimedOut => (
                "the TypeSafe API did not answer in time.".to_owned(),
                "Try again later, or give each call more time with --timeout.".to_owned(),
                6,
            ),
            ScoreError::CannotConnect => (
                "jag cannot connect to the model service.".to_owned(),
                match base {
                    Some(base) => {
                        format!("Check the network connection and {base}, then try again.")
                    }
                    None => "Check the network connection, then try again.".to_owned(),
                },
                6,
            ),
            ScoreError::RateLimited => (
                "the TypeSafe API refused more calls (HTTP 429).".to_owned(),
                "Wait one minute, then try again.".to_owned(),
                6,
            ),
            ScoreError::ServerFailed { status } => (
                format!("the TypeSafe API failed (HTTP {status})."),
                "Try again later.".to_owned(),
                6,
            ),
            ScoreError::Redirect => (
                "the model service sent a redirect.".to_owned(),
                match base {
                    Some(base) => format!("Check {base}."),
                    None => "Update jag, then try again.".to_owned(),
                },
                6,
            ),
            ScoreError::UnexpectedStatus { status } => (
                format!("the TypeSafe API gave HTTP {status}."),
                match (base, model) {
                    (Some(base), Some(model)) => format!("Check {base} and {model}."),
                    (Some(name), None) | (None, Some(name)) => format!("Check {name}."),
                    (None, None) => "Update jag, then try again.".to_owned(),
                },
                6,
            ),
            ScoreError::BadResponse => (
                "the TypeSafe API gave a response that jag cannot read.".to_owned(),
                match base {
                    Some(base) => format!("Check {base}."),
                    None => "Update jag, then try again.".to_owned(),
                },
                6,
            ),
            ScoreError::Stopped => self.stopped_text(),
        }
    }

    /// The texts of the gateway route. The base URL always has a source,
    /// because the default address is the direct route.
    fn gateway_text(self, settings: &ScoreSettings) -> (String, String, u8) {
        let base = settings
            .base_url_origin
            .base_url_name()
            .unwrap_or_else(|| "the base URL".to_owned());
        match self {
            ScoreError::KeyNotAccepted { status } => (
                format!("the gateway did not accept the key (HTTP {status})."),
                if settings.base_url_origin == Origin::Option {
                    "Check that the key is a key of the gateway.".to_owned()
                } else {
                    "Check that the key is a key of the gateway, then run jag --setup.".to_owned()
                },
                4,
            ),
            ScoreError::NoCredit => (
                "the gateway account has no credit (HTTP 402).".to_owned(),
                "Add credit at the gateway, then try again.".to_owned(),
                5,
            ),
            ScoreError::TimedOut => (
                "the gateway did not answer in time.".to_owned(),
                "Try again later, or give each call more time with --timeout.".to_owned(),
                6,
            ),
            ScoreError::CannotConnect => (
                "jag cannot connect to the gateway.".to_owned(),
                format!("Check the network connection and {base}, then try again."),
                6,
            ),
            ScoreError::RateLimited => (
                "the gateway refused more calls (HTTP 429).".to_owned(),
                "Wait one minute, then try again.".to_owned(),
                6,
            ),
            ScoreError::ServerFailed { status } => (
                format!("the gateway failed (HTTP {status})."),
                "Try again later.".to_owned(),
                6,
            ),
            ScoreError::Redirect => (
                "the gateway sent a redirect.".to_owned(),
                format!("Check {base}."),
                6,
            ),
            ScoreError::UnexpectedStatus { status } => (
                format!("the gateway gave HTTP {status}."),
                match settings.model_origin.model_name() {
                    Some(model) => format!("Check {base} and {model}."),
                    None => format!(
                        "Check {base}, and set the model name of the gateway with --model, JAG_MODEL, or config.model."
                    ),
                },
                6,
            ),
            ScoreError::BadResponse => (
                "the gateway gave a response that jag cannot read.".to_owned(),
                format!("Check {base}."),
                6,
            ),
            ScoreError::Stopped => self.stopped_text(),
        }
    }

    fn stopped_text(self) -> (String, String, u8) {
        let failure = Self::stopped();
        (failure.cause, failure.fix, failure.exit_code)
    }

    /// The failure of a stop. It holds no setting, so it is the same on each
    /// route.
    pub fn stopped() -> Failure {
        Failure {
            cause: "the search was stopped.".to_owned(),
            fix: "To get the results, run the search again.".to_owned(),
            exit_code: 130,
        }
    }
}

/// The result of one call, in the slot of its call number. `None` until the
/// call ends, and for a call that never started after a failure.
type Slot = Mutex<Option<Result<Vec<f64>, ScoreError>>>;

/// What the threads of one `score` share.
struct Run<'a> {
    client: Client,
    settings: &'a ScoreSettings,
    end: Instant,
    /// Set after the first failure: no new call and no retry starts.
    stop: AtomicBool,
    /// Set by a Ctrl-C or a closed pipe: the same effect as `stop`.
    external_stop: &'a AtomicBool,
    /// The bill of each answered attempt (FEATURE-20, decision 5).
    usage: &'a Mutex<Usage>,
}

impl Run<'_> {
    /// True when no new call and no retry may start.
    fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst) || self.external_stop.load(Ordering::SeqCst)
    }

    /// One attempt, with the time that is left. No attempt starts after the
    /// deadline.
    fn attempt(&self, body: &str, count: usize) -> Result<Vec<f64>, Attempt> {
        let left = self
            .end
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero());
        let Some(left) = left else {
            return Err(Attempt {
                error: ScoreError::TimedOut,
                retry: false,
            });
        };
        let (result, bill) = self
            .client
            .call(body, count, self.settings.timeout.min(left));
        // A thread that panics must not lose the bill of the others.
        self.usage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add(bill, body.len());
        result
    }

    /// One call, with the one retry of CV-5. The result is `None` when the
    /// stop flag skips the retry. A skipped retry is not a failure: the call
    /// that set the flag has the failure that decides.
    fn call(&self, body: &str, count: usize) -> Option<Result<Vec<f64>, ScoreError>> {
        let first = match self.attempt(body, count) {
            Ok(scores) => return Some(Ok(scores)),
            Err(attempt) => attempt,
        };
        if !first.retry {
            return Some(Err(first.error));
        }
        if self.stopped() {
            return None;
        }
        // No retry starts after the deadline, so a wait that is as long as
        // the time left would only delay the failure.
        let left = self.end.saturating_duration_since(Instant::now());
        if self.settings.retry_wait >= left {
            return Some(Err(first.error));
        }
        std::thread::sleep(self.settings.retry_wait);
        if self.stopped() {
            return None;
        }
        Some(self.attempt(body, count).map_err(|attempt| attempt.error))
    }
}

/// The question and the one result of the readiness check call.
pub const CHECK_QUESTION: &str = "Is jag ready?";
pub const CHECK_PATH: &str = "jag-check";
pub const CHECK_LINE: &[u8] = b"jag is ready";

/// One small call that tests the key, the credit, and the service
/// (FEATURE-02). It has the CV-5 retry and the deadline of `score`. Any score
/// passes: the check tests the service, not the answer.
pub fn probe(key: &ApiKey, settings: &ScoreSettings) -> Result<(), ScoreError> {
    let snippet = Snippet {
        source: Source::File(PathBuf::from(CHECK_PATH)),
        lines: vec![SnippetLine::new(1, CHECK_LINE, true)],
    };
    // The check has no stop flag and no progress (FEATURE-15, decision 16).
    score(
        &[snippet],
        CHECK_QUESTION,
        key,
        settings,
        &AtomicBool::new(false),
        &|_| {},
    )
    .map(|_| ())
}

/// Score each snippet from 0 to 1. The scores are in the order of the input.
///
/// When a call fails, the result is the failure of the lowest call number
/// that failed. A retry that the stop flag skips is not a failure.
///
/// `stop` is the flag of a Ctrl-C or a closed pipe: when it is set, no call and
/// no retry starts, and the result is `ScoreError::Stopped`. `on_call_end`
/// gets the count of results of each call that ends with scores.
pub fn score(
    snippets: &[Snippet],
    question: &str,
    key: &ApiKey,
    settings: &ScoreSettings,
    stop: &AtomicBool,
    on_call_end: &(dyn Fn(usize) + Sync),
) -> Result<Vec<f64>, ScoreError> {
    let usage = Mutex::new(Usage::default());
    score_with_usage(snippets, question, key, settings, stop, on_call_end, &usage)
}

/// `score`, and each answered attempt adds its bill to `usage`, also an
/// attempt that ends after another call failed (FEATURE-20, decision 5).
pub fn score_with_usage(
    snippets: &[Snippet],
    question: &str,
    key: &ApiKey,
    settings: &ScoreSettings,
    stop: &AtomicBool,
    on_call_end: &(dyn Fn(usize) + Sync),
    usage: &Mutex<Usage>,
) -> Result<Vec<f64>, ScoreError> {
    let batches = prompt::split_batches(snippets, question);
    let run = Run {
        client: Client::new(key, settings),
        settings,
        end: Instant::now() + settings.deadline,
        stop: AtomicBool::new(false),
        external_stop: stop,
        usage,
    };
    let next = AtomicUsize::new(0);
    let slots: Vec<Slot> = batches.iter().map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..batches.len().min(MAX_THREADS) {
            scope.spawn(|| {
                while !run.stopped() {
                    let number = next.fetch_add(1, Ordering::SeqCst);
                    let Some(batch) = batches.get(number) else {
                        break;
                    };
                    let body = prompt::request_body(
                        &settings.model,
                        question,
                        &snippets[batch.range.clone()],
                    );
                    let Some(result) = run.call(&body, batch.range.len()) else {
                        continue;
                    };
                    match &result {
                        Ok(_) => on_call_end(batch.range.len()),
                        Err(_) => run.stop.store(true, Ordering::SeqCst),
                    }
                    if let Ok(mut slot) = slots[number].lock() {
                        *slot = Some(result);
                    }
                }
            });
        }
    });

    if stop.load(Ordering::SeqCst) {
        return Err(ScoreError::Stopped);
    }
    let mut scores = Vec::with_capacity(snippets.len());
    for slot in slots {
        match slot.into_inner().ok().flatten() {
            Some(Ok(batch_scores)) => scores.extend(batch_scores),
            Some(Err(error)) => return Err(error),
            // A call that never started: a call with a higher number failed,
            // so go on to find that failure.
            None => {}
        }
    }
    // A missing score with no failure cannot happen. If it does, a wrong
    // order of the results is worse than a failure.
    if scores.len() == snippets.len() {
        Ok(scores)
    } else {
        Err(ScoreError::BadResponse)
    }
}

/// Sort the results by score, highest first. Equal scores go by source (byte
/// order of the label), then by the first line (decision 28).
pub fn rank(snippets: Vec<Snippet>, scores: Vec<f64>) -> Vec<(Snippet, f64)> {
    let mut ranked: Vec<(Snippet, f64)> = snippets.into_iter().zip(scores).collect();
    ranked.sort_by(|(a, a_score), (b, b_score)| {
        b_score
            .total_cmp(a_score)
            .then_with(|| a.source.label().cmp(b.source.label()))
            .then_with(|| a.first_line().cmp(&b.first_line()))
    });
    ranked
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::Times;
    use crate::search::snippet::{SnippetLine, Source};

    fn snippet(path: &str, first: u64) -> Snippet {
        Snippet {
            source: Source::File(PathBuf::from(path)),
            lines: vec![SnippetLine::new(first, b"x", true)],
        }
    }

    fn order(ranked: &[(Snippet, f64)]) -> Vec<(String, u64)> {
        ranked
            .iter()
            .map(|(snippet, _)| {
                (
                    String::from_utf8_lossy(snippet.source.label()).into_owned(),
                    snippet.first_line(),
                )
            })
            .collect()
    }

    #[test]
    fn rank_puts_the_highest_score_first() {
        let ranked = rank(
            vec![snippet("a", 1), snippet("b", 1), snippet("c", 1)],
            vec![0.1, 0.9, 0.3],
        );
        assert_eq!(
            order(&ranked),
            [("b".into(), 1), ("c".into(), 1), ("a".into(), 1)]
        );
        assert_eq!(ranked[0].1, 0.9);
    }

    #[test]
    fn equal_scores_go_by_path_then_by_the_first_line() {
        let ranked = rank(
            vec![
                snippet("b", 1),
                snippet("a", 20),
                snippet("a", 3),
                snippet("B", 9),
            ],
            vec![0.5, 0.5, 0.5, 0.5],
        );
        assert_eq!(
            order(&ranked),
            [
                ("B".into(), 9),
                ("a".into(), 3),
                ("a".into(), 20),
                ("b".into(), 1)
            ]
        );
    }

    #[test]
    fn standard_input_goes_before_a_file_when_the_scores_are_equal() {
        let mut input = snippet("z", 1);
        input.source = Source::StandardInput;
        let ranked = rank(vec![snippet("a.txt", 1), input], vec![0.5, 0.5]);
        assert_eq!(
            order(&ranked),
            [("(standard input)".into(), 1), ("a.txt".into(), 1)]
        );
    }

    #[test]
    fn each_failure_has_its_exit_code_and_no_value() {
        let cases = [
            (ScoreError::KeyNotAccepted { status: 401 }, 4),
            (ScoreError::KeyNotAccepted { status: 403 }, 4),
            (ScoreError::NoCredit, 5),
            (ScoreError::TimedOut, 6),
            (ScoreError::CannotConnect, 6),
            (ScoreError::RateLimited, 6),
            (ScoreError::ServerFailed { status: 503 }, 6),
            (ScoreError::Redirect, 6),
            (ScoreError::UnexpectedStatus { status: 422 }, 6),
            (ScoreError::BadResponse, 6),
            (ScoreError::Stopped, 130),
        ];
        for route in [Route::Direct, Route::Gateway] {
            let settings = settings_of(route, Origin::Variable, Origin::Option);
            for (error, code) in cases.clone() {
                let failure = error.failure(&settings);
                assert_eq!(failure.exit_code, code);
                assert!(!failure.cause.is_empty() && !failure.fix.is_empty());
                // No text holds an address or a model name.
                let text = format!("{}{}", failure.cause, failure.fix);
                assert!(!text.contains("example.test") && !text.contains("a-model"));
            }
        }
    }

    fn settings_of(route: Route, base: Origin, model: Origin) -> ScoreSettings {
        let mut settings = ScoreSettings::new(ModelSettings {
            base_url: "https://example.test/v1".into(),
            base_url_origin: base,
            model: "a-model".into(),
            model_origin: model,
            route,
            times: Times::default(),
        });
        settings.route = route;
        settings
    }

    fn texts(error: ScoreError, settings: &ScoreSettings) -> (String, String) {
        let failure = error.failure(settings);
        (failure.cause, failure.fix)
    }

    #[test]
    fn the_direct_route_keeps_the_texts_of_the_service() {
        let settings = settings_of(Route::Direct, Origin::Default, Origin::Default);
        assert_eq!(
            texts(ScoreError::KeyNotAccepted { status: 401 }, &settings),
            (
                "the TypeSafe API did not accept the key (HTTP 401).".to_owned(),
                format!("Check the key at {KEYS_PAGE}, then run jag --setup.")
            )
        );
        assert_eq!(
            texts(ScoreError::UnexpectedStatus { status: 422 }, &settings).1,
            "Update jag, then try again."
        );
    }

    #[test]
    fn the_gateway_route_names_the_gateway_and_the_source_of_the_address() {
        let settings = settings_of(Route::Gateway, Origin::Variable, Origin::Default);
        assert_eq!(
            texts(ScoreError::KeyNotAccepted { status: 401 }, &settings),
            (
                "the gateway did not accept the key (HTTP 401).".to_owned(),
                "Check that the key is a key of the gateway, then run jag --setup.".to_owned()
            )
        );
        assert_eq!(
            texts(ScoreError::CannotConnect, &settings).1,
            "Check the network connection and JAG_BASE_URL, then try again."
        );
        assert!(
            texts(ScoreError::UnexpectedStatus { status: 404 }, &settings)
                .1
                .contains("set the model name of the gateway")
        );
        let option = settings_of(Route::Gateway, Origin::Option, Origin::Option);
        assert_eq!(
            texts(ScoreError::KeyNotAccepted { status: 401 }, &option).1,
            "Check that the key is a key of the gateway."
        );
        assert_eq!(
            texts(ScoreError::UnexpectedStatus { status: 404 }, &option).1,
            "Check --base-url and --model."
        );
    }

    /// Each cause and each fix of one route, for the origins of the settings.
    fn all_texts(settings: &ScoreSettings) -> Vec<(String, String)> {
        [
            ScoreError::KeyNotAccepted { status: 401 },
            ScoreError::NoCredit,
            ScoreError::TimedOut,
            ScoreError::CannotConnect,
            ScoreError::RateLimited,
            ScoreError::ServerFailed { status: 503 },
            ScoreError::Redirect,
            ScoreError::UnexpectedStatus { status: 422 },
            ScoreError::BadResponse,
        ]
        .into_iter()
        .map(|error| texts(error, settings))
        .collect()
    }

    fn pairs(rows: &[(&str, &str)]) -> Vec<(String, String)> {
        rows.iter()
            .map(|(cause, fix)| ((*cause).to_owned(), (*fix).to_owned()))
            .collect()
    }

    #[test]
    fn the_direct_route_with_no_source_has_these_texts() {
        let settings = settings_of(Route::Direct, Origin::Default, Origin::Default);
        let add_credit = format!("Add credit at {CONSOLE_PAGE}, then try again.");
        let check_key = format!("Check the key at {KEYS_PAGE}, then run jag --setup.");
        assert_eq!(
            all_texts(&settings),
            pairs(&[
                (
                    "the TypeSafe API did not accept the key (HTTP 401).",
                    &check_key
                ),
                (
                    "the TypeSafe account has no credit (HTTP 402).",
                    &add_credit
                ),
                (
                    "the TypeSafe API did not answer in time.",
                    "Try again later, or give each call more time with --timeout."
                ),
                (
                    "jag cannot connect to the model service.",
                    "Check the network connection, then try again."
                ),
                (
                    "the TypeSafe API refused more calls (HTTP 429).",
                    "Wait one minute, then try again."
                ),
                ("the TypeSafe API failed (HTTP 503).", "Try again later."),
                (
                    "the model service sent a redirect.",
                    "Update jag, then try again."
                ),
                (
                    "the TypeSafe API gave HTTP 422.",
                    "Update jag, then try again."
                ),
                (
                    "the TypeSafe API gave a response that jag cannot read.",
                    "Update jag, then try again."
                ),
            ])
        );
    }

    #[test]
    fn the_direct_route_names_the_source_of_the_settings() {
        let settings = settings_of(Route::Direct, Origin::Variable, Origin::Variable);
        let rows = all_texts(&settings);
        assert_eq!(
            rows[3].1,
            "Check the network connection and JAG_BASE_URL, then try again."
        );
        assert_eq!(rows[6].1, "Check JAG_BASE_URL.");
        assert_eq!(rows[7].1, "Check JAG_BASE_URL and JAG_MODEL.");
        assert_eq!(rows[8].1, "Check JAG_BASE_URL.");
        let settings = settings_of(Route::Direct, Origin::Option, Origin::Default);
        let rows = all_texts(&settings);
        assert_eq!(
            rows[3].1,
            "Check the network connection and --base-url, then try again."
        );
        assert_eq!(rows[6].1, "Check --base-url.");
        assert_eq!(rows[7].1, "Check --base-url.");
    }

    #[test]
    fn the_gateway_route_has_these_texts() {
        let settings = settings_of(Route::Gateway, Origin::Variable, Origin::Variable);
        assert_eq!(
            all_texts(&settings),
            pairs(&[
                (
                    "the gateway did not accept the key (HTTP 401).",
                    "Check that the key is a key of the gateway, then run jag --setup."
                ),
                (
                    "the gateway account has no credit (HTTP 402).",
                    "Add credit at the gateway, then try again."
                ),
                (
                    "the gateway did not answer in time.",
                    "Try again later, or give each call more time with --timeout."
                ),
                (
                    "jag cannot connect to the gateway.",
                    "Check the network connection and JAG_BASE_URL, then try again."
                ),
                (
                    "the gateway refused more calls (HTTP 429).",
                    "Wait one minute, then try again."
                ),
                ("the gateway failed (HTTP 503).", "Try again later."),
                ("the gateway sent a redirect.", "Check JAG_BASE_URL."),
                (
                    "the gateway gave HTTP 422.",
                    "Check JAG_BASE_URL and JAG_MODEL."
                ),
                (
                    "the gateway gave a response that jag cannot read.",
                    "Check JAG_BASE_URL."
                ),
            ])
        );
        let option = settings_of(Route::Gateway, Origin::Option, Origin::Option);
        let rows = all_texts(&option);
        assert_eq!(rows[0].1, "Check that the key is a key of the gateway.");
        assert_eq!(
            rows[3].1,
            "Check the network connection and --base-url, then try again."
        );
        assert_eq!(rows[7].1, "Check --base-url and --model.");
        let file = Origin::ConfigFile(PathBuf::from("/h/config.yml"));
        let from_file = settings_of(Route::Gateway, file.clone(), file);
        let rows = all_texts(&from_file);
        assert_eq!(rows[6].1, "Check config.base-url in /h/config.yml.");
        assert_eq!(
            rows[7].1,
            "Check config.base-url in /h/config.yml and config.model in /h/config.yml."
        );
    }

    #[test]
    fn a_stop_has_the_same_text_on_each_route() {
        let direct = settings_of(Route::Direct, Origin::Default, Origin::Default);
        let gateway = settings_of(Route::Gateway, Origin::Option, Origin::Option);
        assert_eq!(
            ScoreError::Stopped.failure(&direct),
            ScoreError::Stopped.failure(&gateway)
        );
        assert_eq!(ScoreError::Stopped.failure(&direct).exit_code, 130);
    }

    fn deadline(timeout: u64, retry_wait: u64) -> Duration {
        let times = Times {
            timeout: Duration::from_secs(timeout),
            retry_wait: Duration::from_secs(retry_wait),
        };
        ScoreSettings::new(ModelSettings {
            base_url: "u".into(),
            base_url_origin: Origin::Default,
            model: "m".into(),
            model_origin: Origin::Default,
            route: Route::Direct,
            times,
        })
        .deadline
    }

    #[test]
    fn the_deadline_is_two_timeouts_and_one_wait() {
        // Compared with the formula, not with a fixed 10 s (Q-as).
        for (timeout, retry_wait) in [(4, 1), (10, 5), (60, 60), (1, 1)] {
            assert_eq!(
                deadline(timeout, retry_wait),
                Duration::from_secs(2 * timeout + retry_wait)
            );
        }
        assert_eq!(deadline(4, 1), Duration::from_secs(9));
        assert_eq!(deadline(10, 5), Duration::from_secs(25));
        assert_eq!(deadline(60, 60), Duration::from_secs(180));
    }

    #[test]
    fn the_settings_hold_the_times() {
        let settings = settings_of(Route::Direct, Origin::Default, Origin::Default);
        assert_eq!(settings.timeout, Duration::from_secs(4));
        assert_eq!(settings.retry_wait, Duration::from_secs(1));
    }
}
