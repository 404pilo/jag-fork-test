//! The `ureq` agent, one call, and the map from a status to a failure.
//!
//! Only this code calls `ApiKey::expose` (CV-6). No failure holds the body of
//! the response, the error text of `ureq`, the address, or the key.

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig};

use super::cost::Bill;
use super::{ScoreError, ScoreSettings};
use crate::config::ApiKey;

/// The failure of one attempt, and whether CV-5 tries it again.
#[derive(Debug)]
pub(super) struct Attempt {
    pub error: ScoreError,
    pub retry: bool,
}

impl Attempt {
    fn retry(error: ScoreError) -> Self {
        Self { error, retry: true }
    }

    fn stop(error: ScoreError) -> Self {
        Self {
            error,
            retry: false,
        }
    }
}

#[derive(Deserialize)]
struct Response {
    answers: HashMap<String, Answer>,
}

#[derive(Deserialize)]
struct Answer {
    noul: f64,
}

/// One agent for the search, with the address and the key.
pub(super) struct Client {
    agent: Agent,
    url: String,
    authorization: String,
}

impl Client {
    pub fn new(key: &ApiKey, settings: &ScoreSettings) -> Self {
        let agent = Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .timeout_global(Some(settings.timeout))
            .max_redirects(0)
            .http_status_as_error(false)
            .user_agent(format!("jag/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .new_agent();
        Self {
            agent,
            url: format!("{}/systemone", settings.base_url),
            authorization: format!("Bearer {}", key.expose()),
        }
    }

    /// One call. `count` is the count of scores that the answer must hold. The
    /// bill is `NoAnswer` for each attempt that gets no HTTP 200.
    pub fn call(
        &self,
        body: &str,
        count: usize,
        timeout: Duration,
    ) -> (Result<Vec<f64>, Attempt>, Bill) {
        let sent = self
            .agent
            .post(&self.url)
            .header("Authorization", &self.authorization)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .config()
            .timeout_global(Some(timeout))
            .build()
            .send(body);
        let mut response = match sent {
            Ok(response) => response,
            Err(error) => {
                return (
                    Err(Attempt::retry(map_transport_error(&error))),
                    Bill::NoAnswer,
                );
            }
        };
        let status = response.status().as_u16();
        let refused = |attempt| (Err(attempt), Bill::NoAnswer);
        match status {
            200 => {}
            401 | 403 => return refused(Attempt::stop(ScoreError::KeyNotAccepted { status })),
            402 => return refused(Attempt::stop(ScoreError::NoCredit)),
            429 => return refused(Attempt::retry(ScoreError::RateLimited)),
            500..=599 => return refused(Attempt::retry(ScoreError::ServerFailed { status })),
            300..=399 => return refused(Attempt::stop(ScoreError::Redirect)),
            _ => return refused(Attempt::stop(ScoreError::UnexpectedStatus { status })),
        }
        // The attempt got an answer, so it counts, also when jag cannot read
        // the scores (decisions 5 and 8).
        let bytes = match response.body_mut().read_to_vec() {
            Ok(bytes) => bytes,
            Err(error) => {
                return (
                    Err(Attempt::retry(map_transport_error(&error))),
                    Bill::NoCount,
                );
            }
        };
        let bill = read_input_tokens(&bytes).map_or(Bill::NoCount, Bill::Tokens);
        let scores =
            parse_scores(&bytes, count).ok_or_else(|| Attempt::stop(ScoreError::BadResponse));
        (scores, bill)
    }
}

fn map_transport_error(error: &ureq::Error) -> ScoreError {
    match error {
        ureq::Error::Timeout(_) => ScoreError::TimedOut,
        _ => ScoreError::CannotConnect,
    }
}

/// The count of input tokens in `usage.input_tokens`. `None` when it is
/// missing, `null`, negative, not a whole number, or text, and when the body is
/// not JSON (decisions 6 and 8). It is read apart from the answers.
fn read_input_tokens(bytes: &[u8]) -> Option<u64> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    value.get("usage")?.get("input_tokens")?.as_u64()
}

/// The scores of `s1` to `s<count>`, in order. `None` when an answer is
/// missing or its `noul` is not a number from 0 to 1.
fn parse_scores(bytes: &[u8], count: usize) -> Option<Vec<f64>> {
    let response: Response = serde_json::from_slice(bytes).ok()?;
    (1..=count)
        .map(|n| {
            let noul = response.answers.get(&format!("s{n}"))?.noul;
            (noul.is_finite() && (0.0..=1.0).contains(&noul)).then_some(noul)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scores_come_in_the_order_of_the_questions() {
        let body = br#"{"answers":{"s2":{"type":"noul","noul":0.25},"s1":{"noul":0.75}},"usage":{"input_tokens":3}}"#;
        assert_eq!(parse_scores(body, 2), Some(vec![0.75, 0.25]));
    }

    #[test]
    fn a_body_that_jag_cannot_read_gives_none() {
        assert_eq!(parse_scores(b"not json", 1), None);
        assert_eq!(parse_scores(br#"{"answers":{}}"#, 1), None);
        assert_eq!(parse_scores(br#"{"answers":{"s1":{"noul":1.5}}}"#, 1), None);
        assert_eq!(
            parse_scores(br#"{"answers":{"s1":{"noul":-0.1}}}"#, 1),
            None
        );
        assert_eq!(parse_scores(br#"{"answers":{"s1":{"noul":"x"}}}"#, 1), None);
        assert_eq!(parse_scores(br#"{"answers":{"s1":{}}}"#, 1), None);
        assert_eq!(parse_scores(br#"{"answers":{"s1":{"noul":0.5}}}"#, 2), None);
    }

    #[test]
    fn the_count_of_input_tokens_is_a_whole_number() {
        let read = |text: &str| read_input_tokens(text.as_bytes());
        assert_eq!(read(r#"{"usage":{"input_tokens":24000}}"#), Some(24000));
        assert_eq!(read(r#"{"usage":{"input_tokens":0}}"#), Some(0));
        assert_eq!(read(r#"{"answers":{}}"#), None);
        assert_eq!(read(r#"{"usage":{}}"#), None);
        assert_eq!(read(r#"{"usage":{"input_tokens":null}}"#), None);
        assert_eq!(read(r#"{"usage":{"input_tokens":-1}}"#), None);
        assert_eq!(read(r#"{"usage":{"input_tokens":2.5}}"#), None);
        assert_eq!(read(r#"{"usage":{"input_tokens":"12"}}"#), None);
        assert_eq!(read("not json"), None);
    }

    #[test]
    fn the_edge_values_are_scores() {
        let body = br#"{"answers":{"s1":{"noul":0},"s2":{"noul":1}}}"#;
        assert_eq!(parse_scores(body, 2), Some(vec![0.0, 1.0]));
    }
}
