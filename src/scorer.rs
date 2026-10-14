//! C-scorer: the model calls that score each result (STORY-13, STORY-15).
//!
//! `score` sends the results to the model service in calls of 32 results or
//! less, 6 calls in parallel, and gives one score for each result.

mod client;
pub mod prompt;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::config::{ApiKey, KEYS_PAGE};
use crate::failure::Failure;
use crate::search::snippet::Snippet;
use client::{Attempt, Client};

/// The version of the scoring prompt (ADR-0004).
pub const PROMPT_VERSION: &str = "jag-p1";
/// A top score below this is "no good answer" (ADR-0003).
pub const THRESHOLD: f64 = 0.4;
/// Before FEATURE-06, jag scores this many results at most (ADR-0002).
pub const MAX_SCORED: usize = 200;
/// The calls that run at the same time.
const MAX_THREADS: usize = 6;

/// The address, the model, and the times of CV-5.
#[derive(Clone, Debug)]
pub struct ScoreSettings {
    pub base_url: String,
    pub model: String,
    /// The time of one attempt.
    pub timeout: Duration,
    /// The wait before the one retry.
    pub retry_wait: Duration,
    /// The time for all calls of a search. No call and no retry starts after
    /// it.
    pub deadline: Duration,
}

impl ScoreSettings {
    /// The times of CV-5: 4 seconds for a call, 1 second before the retry,
    /// and 9 seconds for all calls. The deadline is
    /// `2 x timeout + retry_wait`.
    pub fn new(base_url: String, model: String) -> Self {
        let timeout = Duration::from_secs(4);
        let retry_wait = Duration::from_secs(1);
        Self {
            base_url,
            model,
            timeout,
            retry_wait,
            deadline: 2 * timeout + retry_wait,
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
}

impl From<ScoreError> for Failure {
    fn from(error: ScoreError) -> Failure {
        let (cause, exit_code) = match error {
            ScoreError::KeyNotAccepted { status } => (
                format!("the TypeSafe API did not accept the key (HTTP {status})."),
                4,
            ),
            ScoreError::NoCredit => (
                "the TypeSafe account has no credit (HTTP 402).".to_owned(),
                5,
            ),
            ScoreError::TimedOut => ("the TypeSafe API did not answer in time.".to_owned(), 6),
            ScoreError::CannotConnect => ("jag cannot connect to the model service.".to_owned(), 6),
            ScoreError::RateLimited => (
                "the TypeSafe API refused more calls (HTTP 429).".to_owned(),
                6,
            ),
            ScoreError::ServerFailed { status } => {
                (format!("the TypeSafe API failed (HTTP {status})."), 6)
            }
            ScoreError::Redirect => ("the model service sent a redirect.".to_owned(), 6),
            ScoreError::UnexpectedStatus { status } => {
                (format!("the TypeSafe API gave HTTP {status}."), 6)
            }
            ScoreError::BadResponse => (
                "the TypeSafe API gave a response that jag cannot read.".to_owned(),
                6,
            ),
        };
        let fix = match exit_code {
            4 => format!("Check the key at {KEYS_PAGE}, then run jag --setup."),
            5 => "Add credit to the TypeSafe account.".to_owned(),
            _ => "Try again later.".to_owned(),
        };
        Failure {
            cause,
            fix,
            exit_code,
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
}

impl Run<'_> {
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
        self.client
            .call(body, count, self.settings.timeout.min(left))
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
        if self.stop.load(Ordering::SeqCst) {
            return None;
        }
        // No retry starts after the deadline, so a wait that is as long as
        // the time left would only delay the failure.
        let left = self.end.saturating_duration_since(Instant::now());
        if self.settings.retry_wait >= left {
            return Some(Err(first.error));
        }
        std::thread::sleep(self.settings.retry_wait);
        if self.stop.load(Ordering::SeqCst) {
            return None;
        }
        Some(self.attempt(body, count).map_err(|attempt| attempt.error))
    }
}

/// Score each snippet from 0 to 1. The scores are in the order of the input.
///
/// When a call fails, the result is the failure of the lowest call number
/// that failed. A retry that the stop flag skips is not a failure.
pub fn score(
    snippets: &[Snippet],
    question: &str,
    key: &ApiKey,
    settings: &ScoreSettings,
) -> Result<Vec<f64>, ScoreError> {
    let batches = prompt::split_batches(snippets, question);
    let run = Run {
        client: Client::new(key, settings),
        settings,
        end: Instant::now() + settings.deadline,
        stop: AtomicBool::new(false),
    };
    let next = AtomicUsize::new(0);
    let slots: Vec<Slot> = batches.iter().map(|_| Mutex::new(None)).collect();

    std::thread::scope(|scope| {
        for _ in 0..batches.len().min(MAX_THREADS) {
            scope.spawn(|| {
                while !run.stop.load(Ordering::SeqCst) {
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
                    if result.is_err() {
                        run.stop.store(true, Ordering::SeqCst);
                    }
                    if let Ok(mut slot) = slots[number].lock() {
                        *slot = Some(result);
                    }
                }
            });
        }
    });

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

/// Sort the results by score, highest first. Equal scores go by path (byte
/// order), then by the first line (decision 28).
pub fn rank(snippets: Vec<Snippet>, scores: Vec<f64>) -> Vec<(Snippet, f64)> {
    let mut ranked: Vec<(Snippet, f64)> = snippets.into_iter().zip(scores).collect();
    ranked.sort_by(|(a, a_score), (b, b_score)| {
        b_score
            .total_cmp(a_score)
            .then_with(|| {
                a.path
                    .as_os_str()
                    .as_encoded_bytes()
                    .cmp(b.path.as_os_str().as_encoded_bytes())
            })
            .then_with(|| first_line(a).cmp(&first_line(b)))
    });
    ranked
}

fn first_line(snippet: &Snippet) -> u64 {
    snippet.lines.first().map_or(0, |line| line.number)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::search::snippet::SnippetLine;

    fn snippet(path: &str, first: u64) -> Snippet {
        Snippet {
            path: PathBuf::from(path),
            lines: vec![SnippetLine {
                number: first,
                text: b"x".to_vec(),
                is_match: true,
            }],
        }
    }

    fn order(ranked: &[(Snippet, f64)]) -> Vec<(String, u64)> {
        ranked
            .iter()
            .map(|(snippet, _)| (snippet.path.display().to_string(), first_line(snippet)))
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
        ];
        for (error, code) in cases {
            let failure = Failure::from(error);
            assert_eq!(failure.exit_code, code);
            assert!(!failure.cause.is_empty() && !failure.fix.is_empty());
        }
    }

    #[test]
    fn the_standard_deadline_is_two_timeouts_and_one_wait() {
        let settings = ScoreSettings::new("u".into(), "m".into());
        assert_eq!(settings.timeout, Duration::from_secs(4));
        assert_eq!(settings.retry_wait, Duration::from_secs(1));
        assert_eq!(settings.deadline, Duration::from_secs(9));
    }
}
