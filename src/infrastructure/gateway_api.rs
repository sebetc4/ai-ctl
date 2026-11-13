//! llama-swap's HTTP API, over the HTTP/1.0 client of [`super::http`].

use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use super::api_key::KeyFile;
use super::http::{self, copy_stream, request};
use crate::application::ports::{GatewayApi, RunningModel};

/// llama-swap lists its models from memory.
const RUNNING_TIMEOUT: Duration = Duration::from_secs(5);
/// llama-swap answers an unload once every model is stopped: SIGTERM, then
/// SIGKILL after each model's `unloadTimeout`, 10 s by default.
const UNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// How much of an error's body a log stream reads, to quote it.
const MAX_ERROR_BODY: usize = 64 * 1024;

/// The body of `/running`, of which ai-ctl reads each model's ID, state
/// and proxy address.
#[derive(Deserialize)]
struct Running {
    running: Vec<RunningEntry>,
}

#[derive(Deserialize)]
struct RunningEntry {
    model: String,
    state: String,
    #[serde(default)]
    proxy: String,
}

/// The models a `/running` body lists.
pub fn parse_running(body: &[u8]) -> Result<Vec<RunningModel>> {
    let running: Running =
        serde_json::from_slice(body).context("the list of running models is malformed")?;
    Ok(running
        .running
        .into_iter()
        .map(|entry| RunningModel {
            proxy: proxy_address(&entry.proxy),
            id: entry.model,
            state: entry.state,
        })
        .collect())
}

/// The address of a proxy URL such as `http://127.0.0.1:5800`; `None` for
/// any other form.
fn proxy_address(url: &str) -> Option<SocketAddr> {
    url.strip_prefix("http://")?
        .trim_end_matches('/')
        .parse()
        .ok()
}

/// llama-swap's API on the loopback, which every route but `/health` answers
/// only with ai-ctl's API key.
pub struct LlamaSwapApi {
    key: KeyFile,
}

impl LlamaSwapApi {
    pub fn new(key: KeyFile) -> Self {
        Self { key }
    }
}

impl GatewayApi for LlamaSwapApi {
    fn healthy(&self, listen: SocketAddr, timeout: Duration) -> Result<bool> {
        Ok(request(listen, "GET", "/health", None, timeout, None)?
            .is_some_and(|response| response.status == 200))
    }

    fn unload_all(&self, listen: SocketAddr) -> Result<()> {
        let path = "/api/models/unload";
        let key = self.key.read()?;
        http::request_ok(listen, "POST", path, None, UNLOAD_TIMEOUT, key.as_deref())?;
        Ok(())
    }

    fn running(&self, listen: SocketAddr) -> Result<Vec<RunningModel>> {
        let path = "/running";
        let key = self.key.read()?;
        let body = http::request_ok(listen, "GET", path, None, RUNNING_TIMEOUT, key.as_deref())?;
        parse_running(&body).with_context(|| format!("GET http://{listen}{path}"))
    }

    fn stream_log(&self, listen: SocketAddr, model: &str, out: &mut dyn Write) -> Result<()> {
        let path = format!("/logs/stream/{model}");
        // A stream waits as long as the model is silent.
        let key = self.key.read()?;
        let mut stream = http::open(listen, "GET", &path, None, None, key.as_deref())?;
        match copy_stream(&mut stream, out, MAX_ERROR_BODY)
            .with_context(|| format!("GET http://{listen}{path}"))?
        {
            None => Ok(()),
            // llama-swap knows no log of that name.
            Some(response) if response.status == 400 => {
                bail!("the gateway serves no model {model:?}")
            }
            Some(response) => Err(http::unexpected(listen, "GET", &path, &response)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;
    use crate::infrastructure::http::{parse, serve};

    const HEALTH: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-health.http");
    const RUNNING: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-running-empty.http");
    const LOADED: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-running.http");
    /// `/logs/stream/fake` of a gateway whose model `fake` answered one
    /// request, read for two seconds, then disconnected.
    const STREAM: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-logs-stream.http");
    /// `/logs/stream/nope`, for a model the gateway does not have.
    const UNKNOWN: &[u8] =
        include_bytes!("../../tests/fixtures/llama-swap-270-logs-stream-unknown.http");
    const STREAMED: &str = "fake-upstream fake: listening on 18600\n\
         fake-upstream fake: GET /health\n\
         fake-upstream fake: GET /v1/models\n\
         fake-upstream fake: POST /v1/chat/completions\n";
    const UNLOAD: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-unload.http");
    /// What `up` gives a health check.
    const CHECK: Duration = Duration::from_millis(500);

    /// The API, with no key file.
    fn api() -> LlamaSwapApi {
        LlamaSwapApi::new(KeyFile::new("/nonexistent/api-key".into()))
    }

    #[test]
    fn every_call_but_the_health_check_sends_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let api = LlamaSwapApi::new(KeyFile::written(dir.path(), "k-test"));
        let bearer = "\r\nAuthorization: Bearer k-test\r\n";

        let (addr, server) = serve(RUNNING);
        api.running(addr).unwrap();
        assert!(server.join().unwrap().contains(bearer));
        let (addr, server) = serve(UNLOAD);
        api.unload_all(addr).unwrap();
        assert!(server.join().unwrap().contains(bearer));
        let (addr, server) = serve(STREAM);
        api.stream_log(addr, "fake", &mut Vec::new()).unwrap();
        assert!(server.join().unwrap().contains(bearer));
        let (addr, server) = serve(HEALTH);
        assert!(api.healthy(addr, CHECK).unwrap());
        assert!(!server.join().unwrap().contains("Authorization"));
    }

    #[test]
    fn without_a_key_file_no_key_is_sent() {
        let (addr, server) = serve(RUNNING);
        api().running(addr).unwrap();
        assert!(!server.join().unwrap().contains("Authorization"));
    }

    #[test]
    fn an_unusable_key_file_is_an_error_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        let api = LlamaSwapApi::new(KeyFile::written(dir.path(), "two words"));
        let message = format!(
            "{:#}",
            api.running("127.0.0.1:9".parse().unwrap()).unwrap_err()
        );
        assert!(
            message.starts_with(&format!("{}", dir.path().join("api-key").display())),
            "{message}"
        );
    }

    #[test]
    fn the_running_models_are_read_with_their_state() {
        assert_eq!(
            parse_running(&parse(LOADED).unwrap().body).unwrap(),
            [RunningModel {
                id: "fake".into(),
                state: "ready".into(),
                proxy: Some("127.0.0.1:18600".parse().unwrap()),
            }]
        );
        assert_eq!(
            parse_running(&parse(RUNNING).unwrap().body).unwrap(),
            Vec::new()
        );
        assert_eq!(
            parse_running(b"{\"models\":[]}").unwrap_err().to_string(),
            "the list of running models is malformed"
        );
    }

    #[test]
    fn a_proxy_is_an_address_only_in_its_plain_form() {
        assert_eq!(
            proxy_address("http://127.0.0.1:5801/"),
            Some("127.0.0.1:5801".parse().unwrap())
        );
        for url in ["", "https://127.0.0.1:5801", "http://localhost:5801"] {
            assert_eq!(proxy_address(url), None, "{url}");
        }
        let without = br#"{"running":[{"model":"m","state":"starting"}]}"#;
        assert_eq!(parse_running(without).unwrap()[0].proxy, None);
    }

    #[test]
    fn a_models_log_is_streamed_from_the_gateway() {
        let (addr, server) = serve(STREAM);
        let mut out = Vec::new();
        api().stream_log(addr, "fake", &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), STREAMED);
        assert!(
            server
                .join()
                .unwrap()
                .starts_with("GET /logs/stream/fake HTTP/1.0\r\n")
        );
    }

    #[test]
    fn an_unknown_model_has_no_log_stream() {
        let (addr, server) = serve(UNKNOWN);
        let mut out = Vec::new();
        assert_eq!(
            api()
                .stream_log(addr, "nope", &mut out)
                .unwrap_err()
                .to_string(),
            "the gateway serves no model \"nope\""
        );
        server.join().unwrap();
    }

    #[test]
    fn nothing_listening_is_unhealthy() {
        // A child forked while the port is bound would keep it listening.
        let _releasing = crate::infrastructure::command::exclusive();
        let addr = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        assert!(!api().healthy(addr, CHECK).unwrap());
    }

    #[test]
    fn a_healthy_gateway_answers_200() {
        let (addr, server) = serve(HEALTH);
        assert!(api().healthy(addr, CHECK).unwrap());
        assert!(
            server
                .join()
                .unwrap()
                .starts_with("GET /health HTTP/1.0\r\n")
        );
    }
}
