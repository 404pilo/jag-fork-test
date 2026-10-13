//! The `ureq` agent, one call, and the map from a status to a failure.
//!
//! Only this code calls `ApiKey::expose` (CV-6). No failure holds the body of
//! the response, the error text of `ureq`, the address, or the key.

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig};

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

    /// One call. `count` is the count of scores that the answer must hold.
    pub fn call(&self, body: &str, count: usize, timeout: Duration) -> Result<Vec<f64>, Attempt> {
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
            Err(error) => return Err(Attempt::retry(map_transport_error(&error))),
        };
        let status = response.status().as_u16();
        match status {
            200 => {}
            401 | 403 => return Err(Attempt::stop(ScoreError::KeyNotAccepted { status })),
            402 => return Err(Attempt::stop(ScoreError::NoCredit)),
            429 => return Err(Attempt::retry(ScoreError::RateLimited)),
            500..=599 => return Err(Attempt::retry(ScoreError::ServerFailed { status })),
            300..=399 => return Err(Attempt::stop(ScoreError::Redirect)),
            _ => return Err(Attempt::stop(ScoreError::UnexpectedStatus { status })),
        }
        let bytes = response
            .body_mut()
            .read_to_vec()
            .map_err(|error| Attempt::retry(map_transport_error(&error)))?;
        parse_scores(&bytes, count).ok_or_else(|| Attempt::stop(ScoreError::BadResponse))
    }
}

fn map_transport_error(error: &ureq::Error) -> ScoreError {
    match error {
        ureq::Error::Timeout(_) => ScoreError::TimedOut,
        _ => ScoreError::CannotConnect,
    }
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
    fn the_edge_values_are_scores() {
        let body = br#"{"answers":{"s1":{"noul":0},"s2":{"noul":1}}}"#;
        assert_eq!(parse_scores(body, 2), Some(vec![0.0, 1.0]));
    }
}
