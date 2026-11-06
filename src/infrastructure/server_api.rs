//! llama-server's own endpoints, reached through the gateway's
//! `/upstream/<model>/` passthrough over the HTTP/1.0 client of
//! [`super::http`]: the context size from `/props`, the chat template from
//! `/apply-template`, the tokenizer from `/tokenize`, and the counters of
//! speculative decoding from `/metrics`.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::json;

use super::api_key::KeyFile;
use super::chat_api::messages_json;
use super::http;
use crate::application::ports::ModelServer;
use crate::domain::bench::{Drafts, Message, SpecCounters};

/// The longest wait for an answer: llama-swap loads the model before it
/// passes the first request on, within its 120 s health check.
const TIMEOUT: Duration = Duration::from_secs(300);

/// llama-server, through llama-swap on the loopback.
pub struct LlamaServerApi {
    key: KeyFile,
}

impl LlamaServerApi {
    pub fn new(key: KeyFile) -> Self {
        Self { key }
    }

    /// Sends `method` to `endpoint` of `model`'s server, with `json` as its
    /// body, and returns the body of a 200 response.
    fn call(
        &self,
        listen: SocketAddr,
        model: &str,
        method: &str,
        endpoint: &str,
        json: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let path = format!("/upstream/{model}/{endpoint}");
        let key = self.key.read()?;
        http::request_ok(listen, method, &path, json, TIMEOUT, key.as_deref())
    }

    /// Posts `body` as JSON to `endpoint` of `model`'s server.
    fn post(
        &self,
        listen: SocketAddr,
        model: &str,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&body).context("cannot write the request")?;
        self.call(listen, model, "POST", endpoint, Some(&bytes))
    }
}

impl ModelServer for LlamaServerApi {
    fn context_size(&self, listen: SocketAddr, model: &str) -> Result<u32> {
        parse_context_size(&self.call(listen, model, "GET", "props", None)?)
            .with_context(|| format!("GET http://{listen}/upstream/{model}/props"))
    }

    fn template(&self, listen: SocketAddr, model: &str, messages: &[Message]) -> Result<String> {
        let body = json!({"messages": messages_json(messages)});
        parse_template(&self.post(listen, model, "apply-template", body)?)
            .with_context(|| format!("POST http://{listen}/upstream/{model}/apply-template"))
    }

    fn pieces(&self, listen: SocketAddr, model: &str, text: &str) -> Result<Vec<Vec<u8>>> {
        let body = json!({"content": text, "with_pieces": true});
        parse_pieces(&self.post(listen, model, "tokenize", body)?)
            .with_context(|| format!("POST http://{listen}/upstream/{model}/tokenize"))
    }

    fn count(&self, listen: SocketAddr, model: &str, text: &str) -> Result<usize> {
        let body = json!({"content": text});
        parse_count(&self.post(listen, model, "tokenize", body)?)
            .with_context(|| format!("POST http://{listen}/upstream/{model}/tokenize"))
    }

    fn spec_counters(&self, listen: SocketAddr, model: &str) -> Result<SpecCounters> {
        let body = self.call(listen, model, "GET", "metrics", None)?;
        parse_spec_counters(&String::from_utf8_lossy(&body))
            .with_context(|| format!("GET http://{listen}/upstream/{model}/metrics"))
    }
}

/// The `n_ctx` of a `/props` body.
fn parse_context_size(body: &[u8]) -> Result<u32> {
    #[derive(Deserialize)]
    struct Props {
        default_generation_settings: Settings,
    }
    #[derive(Deserialize)]
    struct Settings {
        n_ctx: u32,
    }
    let props: Props =
        serde_json::from_slice(body).context("the server's properties are malformed")?;
    Ok(props.default_generation_settings.n_ctx)
}

/// The prompt of an `/apply-template` body.
fn parse_template(body: &[u8]) -> Result<String> {
    #[derive(Deserialize)]
    struct Template {
        prompt: String,
    }
    let template: Template =
        serde_json::from_slice(body).context("the templated prompt is malformed")?;
    Ok(template.prompt)
}

/// The pieces of a `/tokenize` body asked `with_pieces`: text, or the bytes
/// of a piece that is not valid UTF-8 on its own.
fn parse_pieces(body: &[u8]) -> Result<Vec<Vec<u8>>> {
    #[derive(Deserialize)]
    struct Tokens {
        tokens: Vec<Token>,
    }
    #[derive(Deserialize)]
    struct Token {
        piece: Piece,
    }
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Piece {
        Text(String),
        Bytes(Vec<u8>),
    }
    let tokens: Tokens = serde_json::from_slice(body).context("the tokens are malformed")?;
    Ok(tokens
        .tokens
        .into_iter()
        .map(|token| match token.piece {
            Piece::Text(text) => text.into_bytes(),
            Piece::Bytes(bytes) => bytes,
        })
        .collect())
}

/// How many tokens a `/tokenize` body lists.
fn parse_count(body: &[u8]) -> Result<usize> {
    #[derive(Deserialize)]
    struct Tokens {
        tokens: Vec<u32>,
    }
    let tokens: Tokens = serde_json::from_slice(body).context("the tokens are malformed")?;
    Ok(tokens.tokens.len())
}

const DRAFTED: &str = "llamacpp:spec_decode_num_draft_tokens_total";
const ACCEPTED: &str = "llamacpp:spec_decode_num_accepted_tokens_total";
const STEPS: &str = "llamacpp:spec_decode_num_drafts_total";
const ACCEPTED_AT: &str = "llamacpp:spec_decode_num_accepted_tokens_per_pos_total";

/// The counters of speculative decoding in a `/metrics` body, in
/// Prometheus's text format: one `name value` line each, the accepted
/// tokens labelled by draft position (`{position="0"}`), absent before a
/// first draft.
fn parse_spec_counters(text: &str) -> Result<SpecCounters> {
    let (mut drafted, mut accepted, mut steps) = (None, None, None);
    let mut accepted_at = BTreeMap::new();
    for line in text.lines().filter(|line| !line.starts_with('#')) {
        let Some((name, value)) = line.rsplit_once(' ') else {
            continue;
        };
        let counter = || counter(value).with_context(|| format!("malformed metric {line:?}"));
        match name {
            DRAFTED => drafted = Some(counter()?),
            ACCEPTED => accepted = Some(counter()?),
            STEPS => steps = Some(counter()?),
            _ => {
                let Some(labels) = name.strip_prefix(ACCEPTED_AT) else {
                    continue;
                };
                let position: usize = labels
                    .strip_prefix("{position=\"")
                    .and_then(|rest| rest.strip_suffix("\"}"))
                    .and_then(|position| position.parse().ok())
                    .with_context(|| format!("malformed metric {line:?}"))?;
                accepted_at.insert(position, counter()?);
            }
        }
    }
    let (Some(proposed), Some(accepted), Some(steps)) = (drafted, accepted, steps) else {
        bail!("the server's metrics hold no counter of speculative decoding");
    };
    let positions = accepted_at.keys().next_back().map_or(0, |last| last + 1);
    Ok(SpecCounters {
        drafts: Drafts { proposed, accepted },
        steps,
        accepted_at: (0..positions)
            .map(|position| accepted_at.get(&position).copied().unwrap_or(0))
            .collect(),
    })
}

/// A counter's value: an integer, which llama-server may write in
/// scientific notation from a million on (`1.23457e+06`).
fn counter(text: &str) -> Result<u64> {
    let value: f64 = text.parse()?;
    ensure!(value.is_finite() && value >= 0.0, "not a count");
    Ok(value.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::{parse, serve};

    /// Fixtures of the 27B's server at `b11538`, read through llama-swap.
    const PROPS: &[u8] = include_bytes!("../../tests/fixtures/llama-server-b11538-props.http");
    /// The template of a system message, `Session x.`, and a user message,
    /// `Hello`.
    const TEMPLATE: &[u8] =
        include_bytes!("../../tests/fixtures/llama-server-b11538-apply-template.http");
    /// `fn main() { println!("héllo — ×"); }`, with its pieces.
    const PIECES: &[u8] =
        include_bytes!("../../tests/fixtures/llama-server-b11538-tokenize-pieces.http");
    /// `<|im_start|>user\nHello<|im_end|>`, special tokens parsed.
    const TOKENS: &[u8] = include_bytes!("../../tests/fixtures/llama-server-b11538-tokenize.http");
    /// After a bench run of one repetition, MTP drafting two tokens.
    const METRICS: &[u8] = include_bytes!("../../tests/fixtures/llama-server-b11538-metrics.http");

    fn body(response: &[u8]) -> Vec<u8> {
        parse(response).unwrap().body
    }

    #[test]
    fn the_context_size_comes_from_the_properties() {
        assert_eq!(parse_context_size(&body(PROPS)).unwrap(), 98_304);
        assert_eq!(
            parse_context_size(b"{\"n_ctx\":4096}")
                .unwrap_err()
                .to_string(),
            "the server's properties are malformed"
        );
    }

    #[test]
    fn the_template_gives_the_prompt_ready_for_the_reply() {
        assert_eq!(
            parse_template(&body(TEMPLATE)).unwrap(),
            "<|im_start|>system\nSession x.<|im_end|>\n<|im_start|>user\nHello<|im_end|>\n\
             <|im_start|>assistant\n<think>\n"
        );
    }

    #[test]
    fn pieces_join_into_the_text_they_were_cut_from() {
        let pieces = parse_pieces(&body(PIECES)).unwrap();
        assert_eq!(pieces.len(), 13);
        assert_eq!(
            String::from_utf8(pieces.concat()).unwrap(),
            "fn main() { println!(\"héllo — ×\"); }"
        );
    }

    #[test]
    fn a_piece_that_is_not_text_comes_as_its_bytes() {
        let body =
            br#"{"tokens":[{"id":1,"piece":"h"},{"id":2,"piece":[195]},{"id":3,"piece":[169]}]}"#;
        let pieces = parse_pieces(body).unwrap();
        assert_eq!(pieces, [b"h".to_vec(), vec![0xC3], vec![0xA9]]);
        assert_eq!(String::from_utf8(pieces.concat()).unwrap(), "hé");
    }

    #[test]
    fn special_tokens_count_once() {
        assert_eq!(parse_count(&body(TOKENS)).unwrap(), 5);
        assert!(parse_count(b"{\"tokens\":[{\"id\":1}]}").is_err());
    }

    #[test]
    fn the_counters_of_a_real_server_are_read() {
        let counters = parse_spec_counters(&String::from_utf8(body(METRICS)).unwrap()).unwrap();
        assert_eq!(
            counters,
            SpecCounters {
                drafts: Drafts {
                    proposed: 7_669,
                    accepted: 6_883
                },
                steps: 3_838,
                accepted_at: vec![3_573, 3_310],
            }
        );
    }

    /// The API, with no key file.
    fn api() -> LlamaServerApi {
        LlamaServerApi::new(KeyFile::new("/nonexistent/api-key".into()))
    }

    #[test]
    fn a_call_sends_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let api = LlamaServerApi::new(KeyFile::written(dir.path(), "k-test"));
        let (addr, server) = serve(METRICS);
        api.spec_counters(addr, "qwen3.8-27b").unwrap();
        assert!(
            server
                .join()
                .unwrap()
                .contains("\r\nAuthorization: Bearer k-test\r\n")
        );
    }

    #[test]
    fn metrics_are_read_through_the_models_upstream() {
        let (addr, server) = serve(METRICS);
        let counters = api().spec_counters(addr, "qwen3.8-27b").unwrap();
        assert_eq!(counters.steps, 3_838);
        let request = server.join().unwrap();
        assert!(request.starts_with("GET /upstream/qwen3.8-27b/metrics HTTP/1.0\r\n"));
    }

    #[test]
    fn the_counters_of_speculative_decoding_are_read_from_the_metrics() {
        let text = "\
# HELP llamacpp:tokens_predicted_total Number of generation tokens processed
# TYPE llamacpp:tokens_predicted_total counter
llamacpp:tokens_predicted_total 1.23457e+06
llamacpp:spec_decode_num_draft_tokens_total 1.08e+06
llamacpp:spec_decode_num_accepted_tokens_total 812345
llamacpp:spec_decode_num_drafts_total 360000
llamacpp:predicted_tokens_seconds nan
# TYPE llamacpp:spec_decode_num_accepted_tokens_per_pos_total counter
llamacpp:spec_decode_num_accepted_tokens_per_pos_total{position=\"0\"} 320000
llamacpp:spec_decode_num_accepted_tokens_per_pos_total{position=\"2\"} 212345
";
        assert_eq!(
            parse_spec_counters(text).unwrap(),
            SpecCounters {
                drafts: Drafts {
                    proposed: 1_080_000,
                    accepted: 812_345
                },
                steps: 360_000,
                accepted_at: vec![320_000, 0, 212_345],
            }
        );
    }

    #[test]
    fn metrics_without_speculative_counters_or_with_bad_values_are_refused() {
        assert_eq!(
            parse_spec_counters("llamacpp:prompt_tokens_total 10\n")
                .unwrap_err()
                .to_string(),
            "the server's metrics hold no counter of speculative decoding"
        );
        for line in [
            "llamacpp:spec_decode_num_drafts_total -1",
            "llamacpp:spec_decode_num_drafts_total many",
            "llamacpp:spec_decode_num_accepted_tokens_per_pos_total{pos=\"0\"} 1",
        ] {
            assert_eq!(
                parse_spec_counters(line).unwrap_err().to_string(),
                format!("malformed metric {line:?}")
            );
        }
    }

    #[test]
    fn a_call_goes_to_the_models_upstream() {
        let (addr, server) = serve(TEMPLATE);
        let prompt = api()
            .template(
                addr,
                "qwen3.8-27b",
                &[Message::system("Session x."), Message::user("Hello")],
            )
            .unwrap();
        assert!(prompt.ends_with("<think>\n"));
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /upstream/qwen3.8-27b/apply-template HTTP/1.0\r\n"));
        assert!(request.ends_with(
            r#"{"messages":[{"content":"Session x.","role":"system"},{"content":"Hello","role":"user"}]}"#
        ));
    }

    #[test]
    fn the_tokenizer_is_asked_for_the_pieces_or_for_the_count() {
        let text = "fn main() { println!(\"héllo — ×\"); }";
        let (addr, server) = serve(PIECES);
        let pieces = api().pieces(addr, "qwen3.8-27b", text).unwrap();
        assert_eq!(String::from_utf8(pieces.concat()).unwrap(), text);
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /upstream/qwen3.8-27b/tokenize HTTP/1.0\r\n"));
        assert!(
            request.ends_with(
                r#"{"content":"fn main() { println!(\"héllo — ×\"); }","with_pieces":true}"#
            ),
            "{request}"
        );

        let (addr, server) = serve(TOKENS);
        let prompt = "<|im_start|>user\nHello<|im_end|>";
        assert_eq!(api().count(addr, "qwen3.8-27b", prompt).unwrap(), 5);
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /upstream/qwen3.8-27b/tokenize HTTP/1.0\r\n"));
        assert!(
            request.ends_with(r#"{"content":"<|im_start|>user\nHello<|im_end|>"}"#),
            "{request}"
        );
    }

    #[test]
    fn an_error_answer_names_the_endpoint() {
        let (addr, server) = serve(include_bytes!(
            "../../tests/fixtures/llama-swap-263-upstream-props-unknown-model.http"
        ));
        let error = api().context_size(addr, "nope").unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("GET http://{addr}/upstream/nope/props answered 404: model not found")
        );
        server.join().unwrap();
    }
}
