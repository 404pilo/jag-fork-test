//! A fake model server on 127.0.0.1. It uses only `std` and `serde_json`.
//!
//! It reads each request, keeps it for the test, and replies as the test says.
//! It counts the calls that run at the same time. Each reply has
//! `Connection: close`.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One request that the server got.
#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    /// The names are in lower case.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }

    /// The `path` of each snippet of the request, in order.
    pub fn snippet_paths(&self) -> Vec<String> {
        self.json()["state"]["snippets"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|item| item["path"].as_str().unwrap_or_default().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The `lines` text of each snippet of the request, in order.
    pub fn snippet_texts(&self) -> Vec<String> {
        self.json()["state"]["snippets"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|item| item["lines"].as_str().unwrap_or_default().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// The reply to one request.
#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub delay: Duration,
}

impl Reply {
    pub fn status(status: u16) -> Reply {
        Reply {
            status,
            body: "{}".to_owned(),
            delay: Duration::ZERO,
        }
    }

    pub fn body(mut self, body: &str) -> Reply {
        self.body = body.to_owned();
        self
    }

    pub fn after(mut self, delay: Duration) -> Reply {
        self.delay = delay;
        self
    }

    /// HTTP 200 with a score for each snippet of the request. `score` gets
    /// the `lines` text of the snippet.
    pub fn scores(request: &Request, score: impl Fn(&str) -> f64) -> Reply {
        let mut answers = serde_json::Map::new();
        for (index, text) in request.snippet_texts().iter().enumerate() {
            answers.insert(
                format!("s{}", index + 1),
                serde_json::json!({ "type": "noul", "noul": score(text) }),
            );
        }
        let body = serde_json::json!({
            "model": "jev-1.13.0",
            "answers": answers,
            "usage": { "input_tokens": 1, "output_tokens": 1 },
        });
        Reply {
            status: 200,
            body: body.to_string(),
            delay: Duration::ZERO,
        }
    }
}

#[derive(Default)]
struct State {
    requests: Mutex<Vec<Request>>,
    current: AtomicUsize,
    most: AtomicUsize,
    stopped: AtomicBool,
}

pub struct FakeServer {
    port: u16,
    state: Arc<State>,
}

impl FakeServer {
    /// Start a server. `handler` gives the reply for each request.
    pub fn start<F>(handler: F) -> FakeServer
    where
        F: Fn(&Request) -> Reply + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("address").port();
        let state = Arc::new(State::default());
        let handler = Arc::new(handler);
        let accept_state = Arc::clone(&state);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if accept_state.stopped.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let state = Arc::clone(&accept_state);
                let handler = Arc::clone(&handler);
                thread::spawn(move || serve(stream, &state, handler.as_ref()));
            }
        });
        FakeServer { port, state }
    }

    /// A server that gives each snippet the same score.
    pub fn constant(score: f64) -> FakeServer {
        FakeServer::start(move |request| Reply::scores(request, |_| score))
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// The value for `JAG_BASE_URL`.
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }

    pub fn requests(&self) -> Vec<Request> {
        self.state.requests.lock().expect("lock").clone()
    }

    /// The most calls that ran at the same time.
    pub fn most_at_once(&self) -> usize {
        self.state.most.load(Ordering::SeqCst)
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.state.stopped.store(true, Ordering::SeqCst);
        // Wake the accept loop so that its thread ends.
        let _ = TcpStream::connect(("127.0.0.1", self.port));
    }
}

fn serve(
    mut stream: TcpStream,
    state: &State,
    handler: &(dyn Fn(&Request) -> Reply + Send + Sync),
) {
    let Some(request) = read_request(&mut stream) else {
        return;
    };
    let now = state.current.fetch_add(1, Ordering::SeqCst) + 1;
    state.most.fetch_max(now, Ordering::SeqCst);
    state.requests.lock().expect("lock").push(request.clone());

    let reply = handler(&request);
    thread::sleep(reply.delay);
    let extra = if (300..400).contains(&reply.status) {
        "Location: http://127.0.0.1:1/elsewhere\r\n"
    } else {
        ""
    };
    let head = format!(
        "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n",
        reply.status,
        reply.body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(reply.body.as_bytes());
    let _ = stream.flush();
    state.current.fetch_sub(1, Ordering::SeqCst);
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(position) = data.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        data.extend_from_slice(&chunk[..read]);
    };
    let head = String::from_utf8_lossy(&data[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?.to_owned();
    let path = first.next()?.to_owned();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let length = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    while data.len() < head_end + length {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            return None;
        }
        data.extend_from_slice(&chunk[..read]);
    }
    Some(Request {
        method,
        path,
        headers,
        body: data[head_end..head_end + length].to_vec(),
    })
}
