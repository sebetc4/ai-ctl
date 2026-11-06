//! Chat completions, streamed and timed: the client of the bench, over the
//! HTTP/1.0 client of [`super::http`].
//!
//! It speaks to the gateway, or to a model's own llama-server for the
//! overhead scenario. The clock starts before the connection opens. The
//! first token is the first event whose delta carries content or reasoning;
//! every later one adds the gap since the one before; the request ends at
//! `data: [DONE]`, whether or not the server closes the connection then. The
//! last events carry `usage`, asked for with `stream_options`, and
//! llama-server's `timings`.

use std::io::Read;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::{Value, json};

use super::api_key::KeyFile;
use super::http::{self, Body};
use crate::application::ports::{ChatClient, ChatRequest, Reply};
use crate::domain::bench::{ClientTimings, Drafts, Message, Sample, ServerTimings, Usage};

const PATH: &str = "/v1/chat/completions";
/// The longest wait for a byte. A model loads before its first request is
/// answered, within llama-swap's 120 s health check, and nothing comes
/// meanwhile; during a long prefill, llama-server sends a ping every 30 s
/// (`--sse-ping-interval`).
const TIMEOUT: Duration = Duration::from_secs(300);
/// How much of an error's body is read, to quote it.
const MAX_ERROR_BODY: usize = 64 * 1024;

/// Chat completions over HTTP, OpenAI-compatible, sending ai-ctl's API key.
pub struct OpenAiChat {
    key: KeyFile,
}

impl OpenAiChat {
    pub fn new(key: KeyFile) -> Self {
        Self { key }
    }
}

impl ChatClient for OpenAiChat {
    fn stream(&self, addr: SocketAddr, request: &ChatRequest) -> Result<Reply> {
        let body = request_body(request)?;
        let key = self.key.read()?;
        let start = Instant::now();
        let mut stream = http::open(
            addr,
            "POST",
            PATH,
            Some(&body),
            Some(TIMEOUT),
            key.as_deref(),
        )?;
        read_reply(&mut stream, &mut || start.elapsed())
            .with_context(|| format!("POST http://{addr}{PATH}"))
    }
}

/// `messages` as chat completions take them, an assistant's reasoning in
/// `reasoning_content`.
pub fn messages_json(messages: &[Message]) -> Value {
    messages
        .iter()
        .map(|message| {
            let mut value = json!({"role": message.role.name(), "content": message.content});
            if let Some(reasoning) = &message.reasoning {
                value["reasoning_content"] = json!(reasoning);
            }
            value
        })
        .collect()
}

/// The JSON body of `request`, streamed with its usage.
fn request_body(request: &ChatRequest) -> Result<Vec<u8>> {
    let mut body = json!({
        "model": request.model,
        "messages": messages_json(&request.messages),
        "max_tokens": request.max_tokens,
        "seed": request.seed,
        "stream": true,
        "stream_options": {"include_usage": true},
    });
    if request.ignore_eos {
        body["ignore_eos"] = json!(true);
    }
    if !request.cache_prompt {
        body["cache_prompt"] = json!(false);
    }
    serde_json::to_vec(&body).context("cannot write the request")
}

/// Reads a streamed reply from `stream`, `clock` giving the time since the
/// request was sent.
fn read_reply(stream: &mut dyn Read, clock: &mut dyn FnMut() -> Duration) -> Result<Reply> {
    let (head, early) = http::read_head(stream)?;
    let mut body = Body::new(stream, &head, early);
    if head.status != 200 {
        let text = body.read_to(MAX_ERROR_BODY)?;
        bail!("answered {}: {}", head.status, http::error_message(&text));
    }
    let (mut events, mut reply) = (Events::default(), Stream::default());
    while let Some(bytes) = body.next_bytes()? {
        let at = clock();
        for data in events.feed(&bytes)? {
            if reply.event(&data, at)? {
                return reply.finish(at);
            }
        }
    }
    bail!("the stream ends before [DONE]")
}

/// Splits an event stream into the data of its events: `data` lines joined,
/// comments such as llama-server's pings and other fields left out.
#[derive(Default)]
struct Events {
    line: Vec<u8>,
    data: Option<Vec<u8>>,
}

impl Events {
    /// The data of the events `bytes` completes.
    fn feed(&mut self, bytes: &[u8]) -> Result<Vec<String>> {
        let mut complete = Vec::new();
        for &byte in bytes {
            if byte != b'\n' {
                self.line.push(byte);
                continue;
            }
            let line = std::mem::take(&mut self.line);
            let line = line.strip_suffix(b"\r").unwrap_or(&line);
            if line.is_empty() {
                if let Some(data) = self.data.take() {
                    complete.push(String::from_utf8(data).context("an event is not text")?);
                }
                continue;
            }
            let (field, value) = match line.iter().position(|&b| b == b':') {
                Some(colon) => (&line[..colon], &line[colon + 1..]),
                None => (line, &[][..]),
            };
            if field == b"data" {
                let value = value.strip_prefix(b" ").unwrap_or(value);
                let data = self.data.get_or_insert_with(Vec::new);
                if !data.is_empty() {
                    data.push(b'\n');
                }
                data.extend_from_slice(value);
            }
        }
        Ok(complete)
    }
}

/// A reply as its events arrive.
#[derive(Default)]
struct Stream {
    first: Option<Duration>,
    last: Option<Duration>,
    gaps: Vec<Duration>,
    content: String,
    reasoning: String,
    timings: Option<ServerTimings>,
    usage: Option<Usage>,
}

impl Stream {
    /// Takes the data of one event, arrived `at`; `true` at `[DONE]`.
    fn event(&mut self, data: &str, at: Duration) -> Result<bool> {
        if data == "[DONE]" {
            return Ok(true);
        }
        let chunk: Chunk =
            serde_json::from_str(data).with_context(|| format!("malformed event {data:?}"))?;
        if let Some(error) = chunk.error {
            bail!("the server reported an error: {}", error.message);
        }
        let mut token = false;
        for delta in chunk.choices.into_iter().map(|choice| choice.delta) {
            for (text, into) in [
                (delta.reasoning_content, &mut self.reasoning),
                (delta.content, &mut self.content),
            ] {
                if let Some(text) = text.filter(|text| !text.is_empty()) {
                    into.push_str(&text);
                    token = true;
                }
            }
        }
        if token {
            match self.last {
                None => self.first = Some(at),
                Some(last) => self.gaps.push(at.saturating_sub(last)),
            }
            self.last = Some(at);
        }
        if let Some(timings) = chunk.timings {
            self.timings = Some(timings.into());
        }
        if let Some(usage) = chunk.usage {
            self.usage = Some(Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
            });
        }
        Ok(false)
    }

    /// The reply, its stream ended `total` after the request.
    fn finish(self, total: Duration) -> Result<Reply> {
        let ttft = self
            .first
            .ok_or_else(|| anyhow!("the stream ends without a token"))?;
        Ok(Reply {
            sample: Sample {
                client: ClientTimings {
                    ttft,
                    total,
                    gaps: self.gaps,
                },
                server: self.timings,
                usage: self.usage,
            },
            content: self.content,
            reasoning: self.reasoning,
        })
    }
}

/// An event of a streamed chat completion.
#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    choices: Vec<Choice>,
    error: Option<ErrorDetail>,
    timings: Option<Timings>,
    usage: Option<UsageBody>,
}

#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Delta,
}

#[derive(Deserialize, Default)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

/// llama-server's `timings` (`server_slot_stats::to_json` in
/// `tools/server/server-common.cpp`): the draft counts only when
/// speculative decoding proposed a token.
#[derive(Deserialize)]
struct Timings {
    cache_n: u64,
    prompt_n: u64,
    prompt_ms: f64,
    predicted_n: u64,
    predicted_ms: f64,
    draft_n: Option<u64>,
    draft_n_accepted: Option<u64>,
}

impl From<Timings> for ServerTimings {
    fn from(timings: Timings) -> Self {
        Self {
            cache_n: timings.cache_n,
            prompt_n: timings.prompt_n,
            prompt_ms: timings.prompt_ms,
            predicted_n: timings.predicted_n,
            predicted_ms: timings.predicted_ms,
            drafts: timings.draft_n.map(|proposed| Drafts {
                proposed,
                accepted: timings.draft_n_accepted.unwrap_or(0),
            }),
        }
    }
}

#[derive(Deserialize)]
struct UsageBody {
    prompt_tokens: u64,
    completion_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::{Trickle, serve};

    /// A reply streamed through llama-swap: 24 reasoning tokens, MTP on.
    const GATEWAY: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-263-chat-stream.http");
    /// A reply streamed by llama-server itself, in chunks: 6 content tokens,
    /// 14 prompt tokens from the cache.
    const DIRECT: &[u8] =
        include_bytes!("../../tests/fixtures/llama-server-b11538-chat-stream.http");
    const UNKNOWN: &[u8] =
        include_bytes!("../../tests/fixtures/llama-swap-263-chat-unknown-model.http");
    const BAD_REQUEST: &[u8] =
        include_bytes!("../../tests/fixtures/llama-swap-263-chat-bad-request.http");

    /// A clock that moves 10 ms at each reading.
    fn ticking() -> impl FnMut() -> Duration {
        let mut now = Duration::ZERO;
        move || {
            now += Duration::from_millis(10);
            now
        }
    }

    fn reply_to(bytes: &[u8]) -> Result<Reply> {
        read_reply(&mut Trickle(bytes), &mut ticking())
    }

    fn request() -> ChatRequest {
        ChatRequest {
            model: "qwen3.8-27b".into(),
            messages: vec![
                Message::system("Be brief."),
                Message::user("Hello"),
                Message::assistant("Hi.", "Greet back."),
                Message::user("Say hello in five words."),
            ],
            max_tokens: 24,
            ignore_eos: true,
            seed: 42,
            cache_prompt: true,
        }
    }

    #[test]
    fn a_reply_through_the_gateway_keeps_its_tokens_and_timings() {
        let reply = reply_to(GATEWAY).unwrap();
        assert!(
            reply
                .reasoning
                .starts_with("The user wants me to say hello in exactly five words.")
        );
        assert!(reply.reasoning.ends_with("\"Hello there, my dear"));
        assert_eq!(reply.content, "");
        let sample = &reply.sample;
        assert_eq!(sample.client.gaps.len(), 23);
        assert!(sample.client.ttft < sample.client.total);
        assert_eq!(
            sample.server,
            Some(ServerTimings {
                cache_n: 0,
                prompt_n: 46,
                prompt_ms: 620.099,
                predicted_n: 24,
                predicted_ms: 402.573,
                drafts: Some(Drafts {
                    proposed: 18,
                    accepted: 13
                }),
            })
        );
        assert_eq!(
            sample.usage,
            Some(Usage {
                prompt_tokens: 46,
                completion_tokens: 24
            })
        );
    }

    #[test]
    fn a_chunked_reply_from_llama_server_reads_alike() {
        let reply = reply_to(DIRECT).unwrap();
        assert_eq!(reply.content, "Hello there, my friend.");
        assert_eq!(reply.reasoning, "");
        assert_eq!(reply.sample.client.gaps.len(), 5);
        let server = reply.sample.server.unwrap();
        assert_eq!((server.cache_n, server.prompt_n), (14, 4));
        assert_eq!(
            server.drafts,
            Some(Drafts {
                proposed: 6,
                accepted: 3
            })
        );
    }

    #[test]
    fn the_clock_marks_the_first_token_and_the_gaps_after_it() {
        let mut stream = Stream::default();
        let at = Duration::from_millis;
        let role = r#"{"choices":[{"delta":{"role":"assistant","content":null}}]}"#;
        assert!(!stream.event(role, at(5)).unwrap());
        for (text, ms) in [("Hel", 120), ("lo", 140), ("", 150), (" you", 175)] {
            let data = format!(r#"{{"choices":[{{"delta":{{"content":"{text}"}}}}]}}"#);
            assert!(!stream.event(&data, at(ms)).unwrap());
        }
        assert!(stream.event("[DONE]", at(180)).unwrap());
        let reply = stream.finish(at(180)).unwrap();
        assert_eq!(reply.content, "Hello you");
        let client = reply.sample.client;
        assert_eq!((client.ttft, client.total), (at(120), at(180)));
        assert_eq!(client.gaps, [at(20), at(35)]);
        assert_eq!(reply.sample.server, None);
    }

    #[test]
    fn pings_comments_and_carriage_returns_are_read_past() {
        let mut events = Events::default();
        let first = events
            .feed(b":\n\nevent: x\r\ndata: {\"a\":\r\ndata:1}\r\n\r\n: ping\n\nda")
            .unwrap();
        assert_eq!(first, ["{\"a\":\n1}"]);
        assert_eq!(
            events.feed(b"ta: [DONE]\n\n").unwrap(),
            ["[DONE]".to_owned()]
        );
    }

    #[test]
    fn an_error_response_gives_its_message() {
        assert_eq!(
            reply_to(UNKNOWN).unwrap_err().to_string(),
            "answered 404: no router for requested model"
        );
        assert_eq!(
            reply_to(BAD_REQUEST).unwrap_err().to_string(),
            "answered 400: Expected 'messages' to be an array"
        );
        let plain = b"HTTP/1.0 502 Bad Gateway\r\nContent-Length: 4\r\n\r\nEOF\n";
        assert_eq!(
            reply_to(plain).unwrap_err().to_string(),
            "answered 502: EOF"
        );
    }

    #[test]
    fn an_error_in_the_stream_ends_it() {
        let stream = b"HTTP/1.0 200 OK\r\n\r\ndata: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\n\
            data: {\"error\":{\"code\":500,\"message\":\"context full\",\"type\":\"server_error\"}}\n\n";
        assert_eq!(
            reply_to(stream).unwrap_err().to_string(),
            "the server reported an error: context full"
        );
    }

    #[test]
    fn a_stream_cut_short_or_without_a_token_is_an_error() {
        let cut = &GATEWAY[..GATEWAY.len() - 20];
        assert_eq!(
            reply_to(cut).unwrap_err().to_string(),
            "the stream ends before [DONE]"
        );
        let empty = b"HTTP/1.0 200 OK\r\n\r\ndata: {\"choices\":[]}\n\ndata: [DONE]\n\n";
        assert_eq!(
            reply_to(empty).unwrap_err().to_string(),
            "the stream ends without a token"
        );
        let garbled = b"HTTP/1.0 200 OK\r\n\r\ndata: {nope\n\n";
        assert_eq!(
            reply_to(garbled).unwrap_err().to_string(),
            "malformed event \"{nope\""
        );
    }

    #[test]
    fn the_request_streams_its_usage_and_sends_reasoning_back() {
        let body: Value = serde_json::from_slice(&request_body(&request()).unwrap()).unwrap();
        assert_eq!(
            body,
            json!({
                "model": "qwen3.8-27b",
                "messages": [
                    {"role": "system", "content": "Be brief."},
                    {"role": "user", "content": "Hello"},
                    {"role": "assistant", "content": "Hi.", "reasoning_content": "Greet back."},
                    {"role": "user", "content": "Say hello in five words."},
                ],
                "max_tokens": 24,
                "seed": 42,
                "stream": true,
                "stream_options": {"include_usage": true},
                "ignore_eos": true,
            })
        );
        let mut free = request();
        free.ignore_eos = false;
        let body: Value = serde_json::from_slice(&request_body(&free).unwrap()).unwrap();
        assert_eq!(body.get("ignore_eos"), None);
        assert_eq!(body.get("cache_prompt"), None);
        let mut fresh = request();
        fresh.cache_prompt = false;
        let body: Value = serde_json::from_slice(&request_body(&fresh).unwrap()).unwrap();
        assert_eq!(body["cache_prompt"], json!(false));
    }

    #[test]
    fn a_request_reaches_the_server_and_times_its_reply() {
        let (addr, server) = serve(GATEWAY);
        let dir = tempfile::tempdir().unwrap();
        let chat = OpenAiChat::new(KeyFile::written(dir.path(), "k-test"));
        let reply = chat.stream(addr, &request()).unwrap();
        assert_eq!(reply.sample.usage.unwrap().completion_tokens, 24);
        let sent = server.join().unwrap();
        assert!(sent.starts_with("POST /v1/chat/completions HTTP/1.0\r\n"));
        assert!(sent.contains("\r\nAuthorization: Bearer k-test\r\n"));
        assert!(sent.contains("\r\nContent-Type: application/json\r\n"));
        assert!(sent.ends_with("\"stream_options\":{\"include_usage\":true}}"));
    }
}
