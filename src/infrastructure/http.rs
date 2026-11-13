//! A minimal HTTP/1.0 client for the servers of the loopback: llama-swap,
//! and the llama-servers it starts. One request per connection; a body runs
//! to its `Content-Length`, to its last chunk, or until the server closes the
//! connection. llama-swap answers HTTP/1.0 without chunks, a log stream
//! included; llama-server answers HTTP/1.1 and chunks a stream even to an
//! HTTP/1.0 request, so a body read as it arrives is decoded. Both listen on
//! the loopback, in plain HTTP, so the standard library's sockets suffice.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde::Deserialize;

/// Connecting to a local port succeeds or is refused at once; longer means
/// a firewall or an address that does not route.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
/// Headers longer than this are not those of a server ai-ctl talks to.
const MAX_HEAD: usize = 64 * 1024;
/// A chunk-size line longer than this is not one.
const MAX_CHUNK_LINE: usize = 1024;

/// A response: its status code and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

/// The status line and headers of a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Head {
    pub status: u16,
    /// The length of the body, when a `Content-Length` gives it.
    pub length: Option<usize>,
    /// Whether the body comes in chunks.
    pub chunked: bool,
}

/// A connection with its request sent, read as the response arrives.
#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    /// How long each read waits at most; `None` for ever.
    timeout: Option<Duration>,
}

impl Read for Connection {
    /// A read that times out, `EAGAIN` on Linux, is told as no answer
    /// within the timeout rather than as a resource temporarily unavailable.
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream
            .read(buffer)
            .map_err(|err| match (err.kind(), self.timeout) {
                (io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut, Some(timeout)) => {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        format!("no answer within {}", duration(timeout)),
                    )
                }
                _ => err,
            })
    }
}

/// `2 s`, or `500 ms` for a duration that is not whole seconds.
fn duration(duration: Duration) -> String {
    if duration.subsec_nanos() == 0 {
        format!("{} s", duration.as_secs())
    } else {
        format!("{} ms", duration.as_millis())
    }
}

/// A connection to `addr`, `None` when nothing listens there; the request
/// sent, with `json` as its body and `key` as a bearer token, waiting
/// `timeout` at most for each read and write, or forever. A `timeout`
/// shorter than [`CONNECT_TIMEOUT`] also bounds the connection.
pub fn send(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json: Option<&[u8]>,
    timeout: Option<Duration>,
    key: Option<&str>,
) -> Result<Option<Connection>> {
    let connect = timeout.map_or(CONNECT_TIMEOUT, |timeout| timeout.min(CONNECT_TIMEOUT));
    let mut stream = match TcpStream::connect_timeout(&addr, connect) {
        Ok(stream) => stream,
        Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("cannot connect to {addr}")),
    };
    let mut write = || -> io::Result<()> {
        stream.set_read_timeout(timeout)?;
        stream.set_write_timeout(timeout)?;
        let body = json.unwrap_or_default();
        let content_type = if json.is_some() {
            "Content-Type: application/json\r\n"
        } else {
            ""
        };
        let authorization = key.map_or_else(String::new, |key| {
            format!("Authorization: Bearer {key}\r\n")
        });
        let head = format!(
            "{method} {path} HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\n{authorization}{content_type}Content-Length: {}\r\n\r\n",
            body.len()
        );
        stream.write_all(&[head.as_bytes(), body].concat())
    };
    write().with_context(|| format!("{method} http://{addr}{path}"))?;
    Ok(Some(Connection { stream, timeout }))
}

/// Sends `method path` to `addr`, with `json` as its body and `key` as a
/// bearer token, and reads the response within `timeout`; `None` when
/// nothing listens on `addr`.
pub fn request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json: Option<&[u8]>,
    timeout: Duration,
    key: Option<&str>,
) -> Result<Option<Response>> {
    let Some(mut stream) = send(addr, method, path, json, Some(timeout), key)? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .with_context(|| format!("{method} http://{addr}{path}"))?;
    parse(&bytes)
        .map(Some)
        .with_context(|| format!("{method} http://{addr}{path}"))
}

/// The body of a 200 response to `method path` on `addr`, sent as
/// [`request`] sends it; an error when nothing listens, or for any other
/// status, quoting the error body's message.
pub fn request_ok(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json: Option<&[u8]>,
    timeout: Duration,
    key: Option<&str>,
) -> Result<Vec<u8>> {
    let response =
        request(addr, method, path, json, timeout, key)?.ok_or_else(|| nothing_listens(addr))?;
    if response.status != 200 {
        return Err(unexpected(addr, method, path, &response));
    }
    Ok(response.body)
}

/// The connection to `addr` with the request sent, as [`send`] sends it;
/// an error when nothing listens there.
pub fn open(
    addr: SocketAddr,
    method: &str,
    path: &str,
    json: Option<&[u8]>,
    timeout: Option<Duration>,
    key: Option<&str>,
) -> Result<Connection> {
    send(addr, method, path, json, timeout, key)?.ok_or_else(|| nothing_listens(addr))
}

fn nothing_listens(addr: SocketAddr) -> anyhow::Error {
    anyhow!("nothing listens on {addr}")
}

/// The error a response other than 200 gives `method path` on `addr`.
pub fn unexpected(
    addr: SocketAddr,
    method: &str,
    path: &str,
    response: &Response,
) -> anyhow::Error {
    anyhow!(
        "{method} http://{addr}{path} answered {}: {}",
        response.status,
        error_message(&response.body)
    )
}

/// The message of an error body, `{"error": {"message": …}}` as llama-swap,
/// llama-server and other OpenAI-compatible servers write it, or else the
/// body itself.
pub fn error_message(body: &[u8]) -> String {
    #[derive(Deserialize)]
    struct ErrorBody {
        error: ErrorDetail,
    }
    #[derive(Deserialize)]
    struct ErrorDetail {
        message: String,
    }
    serde_json::from_slice::<ErrorBody>(body)
        .map(|body| body.error.message)
        .unwrap_or_else(|_| String::from_utf8_lossy(body).trim().to_owned())
}

/// Where the headers of `bytes` end, before the blank line.
fn head_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

/// Parses the status line and the headers of a response, without the blank
/// line.
fn parse_head(head: &[u8]) -> Result<Head> {
    let head = std::str::from_utf8(head).context("the response headers are not text")?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or_default();
    let status = match status_line.split(' ').collect::<Vec<_>>()[..] {
        [version, code, ..] if version.starts_with("HTTP/1.") => code.parse::<u16>().ok(),
        _ => None,
    }
    .ok_or_else(|| anyhow!("malformed status line {status_line:?}"))?;
    let mut parsed = Head {
        status,
        length: None,
        chunked: false,
    };
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            bail!("malformed header {line:?}");
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            if value.eq_ignore_ascii_case("chunked") {
                parsed.chunked = true;
            } else if !value.eq_ignore_ascii_case("identity") {
                bail!("unexpected transfer encoding {value:?}");
            }
        }
        if name.eq_ignore_ascii_case("content-length") {
            parsed.length = Some(
                value
                    .parse()
                    .map_err(|_| anyhow!("malformed Content-Length {value:?}"))?,
            );
        }
    }
    Ok(parsed)
}

/// Parses a whole HTTP/1.0 response, as llama-swap sends it: a
/// `Content-Length` bounds the body, and chunks are refused.
pub fn parse(bytes: &[u8]) -> Result<Response> {
    let end = head_end(bytes).ok_or_else(|| anyhow!("the response ends before its headers do"))?;
    let head = parse_head(&bytes[..end])?;
    ensure!(
        !head.chunked,
        "unexpected transfer encoding \"chunked\" in an HTTP/1.0 response"
    );
    let mut body = bytes[end + 4..].to_vec();
    if let Some(length) = head.length {
        if body.len() < length {
            bail!(
                "the response ends after {} of its {length} bytes",
                body.len()
            );
        }
        body.truncate(length);
    }
    Ok(Response {
        status: head.status,
        body,
    })
}

/// Reads the head of a response from `stream`: the head, and the bytes of
/// the body read with it.
pub fn read_head(stream: &mut dyn Read) -> Result<(Head, Vec<u8>)> {
    let mut received = Vec::new();
    let mut buffer = [0; 8192];
    let end = loop {
        if let Some(end) = head_end(&received) {
            break end;
        }
        ensure!(
            received.len() <= MAX_HEAD,
            "the response headers exceed {MAX_HEAD} bytes"
        );
        let read = stream
            .read(&mut buffer)
            .context("cannot read the response")?;
        ensure!(read > 0, "the response ends before its headers do");
        received.extend_from_slice(&buffer[..read]);
    };
    let head = parse_head(&received[..end])?;
    Ok((head, received.split_off(end + 4)))
}

/// The body of a response, read as it arrives.
pub struct Body<'a> {
    stream: &'a mut dyn Read,
    /// Bytes read with the head, not given yet.
    early: Vec<u8>,
    framing: Framing,
}

enum Framing {
    /// Until the server closes the connection.
    Close,
    /// This many bytes still to come.
    Length(usize),
    Chunked(Dechunker),
}

impl<'a> Body<'a> {
    /// The body that follows `head` on `stream`, `early` its first bytes.
    pub fn new(stream: &'a mut dyn Read, head: &Head, early: Vec<u8>) -> Self {
        let framing = if head.chunked {
            Framing::Chunked(Dechunker::default())
        } else {
            head.length.map_or(Framing::Close, Framing::Length)
        };
        Self {
            stream,
            early,
            framing,
        }
    }

    /// The next bytes of the body, as they arrive; `None` at its end.
    pub fn next_bytes(&mut self) -> Result<Option<Vec<u8>>> {
        loop {
            if matches!(self.framing, Framing::Length(0))
                || matches!(&self.framing, Framing::Chunked(chunks) if chunks.is_done())
            {
                return Ok(None);
            }
            let mut raw = std::mem::take(&mut self.early);
            if raw.is_empty() {
                let mut buffer = [0; 8192];
                let read = self
                    .stream
                    .read(&mut buffer)
                    .context("cannot read the response")?;
                raw.extend_from_slice(&buffer[..read]);
            }
            let end = raw.is_empty();
            match &mut self.framing {
                Framing::Close if end => return Ok(None),
                Framing::Close => return Ok(Some(raw)),
                Framing::Length(remaining) => {
                    ensure!(!end, "the response ends {remaining} bytes before its end");
                    raw.truncate(*remaining);
                    *remaining -= raw.len();
                    return Ok(Some(raw));
                }
                Framing::Chunked(chunks) => {
                    ensure!(!end, "the response ends inside its chunks");
                    let mut data = Vec::new();
                    chunks.feed(&raw, &mut data)?;
                    if !data.is_empty() {
                        return Ok(Some(data));
                    }
                }
            }
        }
    }

    /// The rest of the body, `limit` bytes at most.
    pub fn read_to(&mut self, limit: usize) -> Result<Vec<u8>> {
        let mut body = Vec::new();
        while body.len() < limit {
            match self.next_bytes()? {
                Some(bytes) => body.extend_from_slice(&bytes),
                None => break,
            }
        }
        body.truncate(limit);
        Ok(body)
    }
}

/// Decodes a chunked body as it arrives.
#[derive(Debug, Default)]
pub struct Dechunker {
    state: Chunk,
    /// The line being read: a chunk's size, the end of its data, a trailer.
    line: Vec<u8>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum Chunk {
    #[default]
    Size,
    /// This many bytes of data still to come.
    Data(usize),
    /// The line break that ends a chunk's data.
    DataEnd,
    /// The trailer after the last chunk, until a blank line.
    Trailer,
    Done,
}

impl Dechunker {
    /// Appends the data that `input` carries to `out`.
    pub fn feed(&mut self, mut input: &[u8], out: &mut Vec<u8>) -> Result<()> {
        while !input.is_empty() && self.state != Chunk::Done {
            if let Chunk::Data(remaining) = &mut self.state {
                let taken = input.len().min(*remaining);
                out.extend_from_slice(&input[..taken]);
                input = &input[taken..];
                *remaining -= taken;
                if *remaining == 0 {
                    self.state = Chunk::DataEnd;
                }
                continue;
            }
            let Some(newline) = input.iter().position(|&b| b == b'\n') else {
                self.line.extend_from_slice(input);
                ensure!(
                    self.line.len() <= MAX_CHUNK_LINE,
                    "malformed chunk: a line exceeds {MAX_CHUNK_LINE} bytes"
                );
                return Ok(());
            };
            self.line.extend_from_slice(&input[..newline]);
            input = &input[newline + 1..];
            let line = std::mem::take(&mut self.line);
            let line = line.strip_suffix(b"\r").unwrap_or(&line);
            self.state = match self.state {
                Chunk::Size => match chunk_size(line)? {
                    0 => Chunk::Trailer,
                    size => Chunk::Data(size),
                },
                Chunk::DataEnd if line.is_empty() => Chunk::Size,
                Chunk::DataEnd => bail!("malformed chunk: its data runs past its size"),
                Chunk::Trailer if line.is_empty() => Chunk::Done,
                _ => Chunk::Trailer,
            };
        }
        Ok(())
    }

    /// Whether the last chunk and the trailer have been read.
    pub fn is_done(&self) -> bool {
        self.state == Chunk::Done
    }
}

/// The size a chunk-size line gives, in hexadecimal before any extension.
fn chunk_size(line: &[u8]) -> Result<usize> {
    let text = String::from_utf8_lossy(line);
    let digits = text.split(';').next().unwrap_or_default().trim();
    usize::from_str_radix(digits, 16).map_err(|_| anyhow!("malformed chunk size {text:?}"))
}

/// Reads a response from `stream`, copying its body to `out` as it comes
/// until the server closes the connection. `None` once a 200 response is
/// copied; any other response is returned instead, its body cut at
/// `error_limit` bytes.
pub fn copy_stream(
    stream: &mut dyn Read,
    out: &mut dyn Write,
    error_limit: usize,
) -> Result<Option<Response>> {
    let (head, early) = read_head(stream)?;
    let mut body = Body::new(stream, &head, early);
    if head.status != 200 {
        let body = body.read_to(error_limit)?;
        return Ok(Some(Response {
            status: head.status,
            body,
        }));
    }
    while let Some(bytes) = body.next_bytes()? {
        out.write_all(&bytes)
            .and_then(|()| out.flush())
            .context("cannot write the stream")?;
    }
    Ok(None)
}

/// A server for tests: it answers one request on a free port of the
/// loopback with `response`, and returns the request.
#[cfg(test)]
pub fn serve(response: &'static [u8]) -> (SocketAddr, std::thread::JoinHandle<String>) {
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (mut request, mut buffer) = (Vec::new(), [0; 4096]);
        let expected = loop {
            if let Some(end) = head_end(&request) {
                let head = String::from_utf8_lossy(&request[..end]).to_lowercase();
                let length = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .map_or(0, |value| value.trim().parse::<usize>().unwrap());
                break end + 4 + length;
            }
            let n = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..n]);
        };
        while request.len() < expected {
            let n = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..n]);
        }
        stream.write_all(response).unwrap();
        String::from_utf8(request).unwrap()
    });
    (addr, server)
}

/// A reader for tests that gives `bytes` a few at a time, as a socket may.
#[cfg(test)]
pub struct Trickle<'a>(pub &'a [u8]);

#[cfg(test)]
impl Read for Trickle<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let n = self.0.len().min(buffer.len()).min(3);
        buffer[..n].copy_from_slice(&self.0[..n]);
        self.0 = &self.0[n..];
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;
    use crate::application::fakes::{ClosedPipe, broken_pipe};

    const HEALTH: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-health.http");
    const RUNNING: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-running-empty.http");
    const UNLOAD: &[u8] = include_bytes!("../../tests/fixtures/llama-swap-270-unload.http");
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
    /// `GET /upstream/nope/props`, for a model llama-swap does not know.
    const NO_MODEL: &[u8] =
        include_bytes!("../../tests/fixtures/llama-swap-270-upstream-props-unknown-model.http");
    /// A streamed chat completion from llama-server, in chunks.
    const CHUNKED: &[u8] =
        include_bytes!("../../tests/fixtures/llama-server-b11538-chat-stream.http");

    #[test]
    fn a_health_response_parses() {
        let response = parse(HEALTH).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"OK");
    }

    #[test]
    fn an_unload_response_parses() {
        let response = parse(UNLOAD).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"{\"msg\":\"ok\"}\n");
    }

    #[test]
    fn a_body_stops_at_its_content_length() {
        let response = parse(RUNNING).unwrap();
        assert_eq!(response.body, b"{\"running\":[]}\n");
        let mut padded = RUNNING.to_vec();
        padded.extend_from_slice(b"trailing");
        assert_eq!(parse(&padded).unwrap().body, response.body);
    }

    #[test]
    fn a_truncated_or_malformed_response_is_an_error() {
        let cases: [(&[u8], &str); 6] = [
            (&HEALTH[..20], "the response ends before its headers do"),
            (
                &HEALTH[..HEALTH.len() - 1],
                "the response ends after 1 of its 2 bytes",
            ),
            (
                b"SSH-2.0-OpenSSH\r\n\r\n",
                "malformed status line \"SSH-2.0-OpenSSH\"",
            ),
            (
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nOK\r\n0\r\n\r\n",
                "unexpected transfer encoding \"chunked\" in an HTTP/1.0 response",
            ),
            (
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n",
                "unexpected transfer encoding \"gzip\"",
            ),
            (
                b"HTTP/1.0 200 OK\r\nContent-Length: two\r\n\r\nOK",
                "malformed Content-Length \"two\"",
            ),
        ];
        for (bytes, expected) in cases {
            assert_eq!(parse(bytes).unwrap_err().to_string(), expected);
        }
    }

    /// The whole body of `bytes`, read as it arrives.
    fn body_of(bytes: &[u8]) -> Result<Vec<u8>> {
        let mut stream = Trickle(bytes);
        let (head, early) = read_head(&mut stream)?;
        Body::new(&mut stream, &head, early).read_to(usize::MAX)
    }

    #[test]
    fn a_chunked_body_is_decoded_as_it_arrives() {
        let body = String::from_utf8(body_of(CHUNKED).unwrap()).unwrap();
        assert!(body.starts_with("data: {\"choices\":[{\"finish_reason\":null"));
        assert!(body.ends_with("data: [DONE]\n\n"));
        assert!(!body.contains("\r\n"));
    }

    #[test]
    fn a_body_runs_to_its_length_or_to_the_close() {
        let sized = b"HTTP/1.0 200 OK\r\nContent-Length: 2\r\n\r\nOKtrailing";
        assert_eq!(body_of(sized).unwrap(), b"OK");
        let closed = b"HTTP/1.0 200 OK\r\n\r\nall of it";
        assert_eq!(body_of(closed).unwrap(), b"all of it");
        let empty = b"HTTP/1.0 204 No Content\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(body_of(empty).unwrap(), b"");
    }

    #[test]
    fn a_body_cut_short_is_an_error() {
        let sized = b"HTTP/1.0 200 OK\r\nContent-Length: 5\r\n\r\nOK";
        assert_eq!(
            body_of(sized).unwrap_err().to_string(),
            "the response ends 3 bytes before its end"
        );
        assert_eq!(
            body_of(&CHUNKED[..CHUNKED.len() - 5])
                .unwrap_err()
                .to_string(),
            "the response ends inside its chunks"
        );
    }

    #[test]
    fn an_error_body_gives_its_message() {
        let llama_swap = parse(include_bytes!(
            "../../tests/fixtures/llama-swap-270-chat-unknown-model.http"
        ))
        .unwrap()
        .body;
        assert_eq!(error_message(&llama_swap), "no router for requested model");
        assert_eq!(error_message(b"proxy error: EOF\n"), "proxy error: EOF");
    }

    #[test]
    fn chunks_split_anywhere_decode_alike() {
        let encoded = b"4;name=value\r\nWiki\r\n5\r\npedia\r\nE\r\n in\r\n\r\nchunks.\r\n0\r\nExpires: never\r\n\r\n";
        for split in 0..encoded.len() {
            let (mut chunks, mut out) = (Dechunker::default(), Vec::new());
            chunks.feed(&encoded[..split], &mut out).unwrap();
            chunks.feed(&encoded[split..], &mut out).unwrap();
            assert_eq!(out, b"Wikipedia in\r\n\r\nchunks.", "split at {split}");
            assert!(chunks.is_done());
        }
    }

    #[test]
    fn malformed_chunks_are_errors() {
        let cases: [(&[u8], &str); 3] = [
            (b"zz\r\n", "malformed chunk size \"zz\""),
            (
                b"2\r\nOKK\r\n",
                "malformed chunk: its data runs past its size",
            ),
            (&[b'1'; 2000], "malformed chunk: a line exceeds 1024 bytes"),
        ];
        for (bytes, expected) in cases {
            let error = Dechunker::default()
                .feed(bytes, &mut Vec::new())
                .unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn a_stream_is_copied_until_the_server_closes_it() {
        let mut out = Vec::new();
        assert_eq!(
            copy_stream(&mut Trickle(STREAM), &mut out, 1024).unwrap(),
            None
        );
        assert_eq!(String::from_utf8(out).unwrap(), STREAMED);
    }

    #[test]
    fn a_stream_refused_gives_its_response() {
        let mut out = Vec::new();
        let response = copy_stream(&mut Trickle(UNKNOWN), &mut out, 1024)
            .unwrap()
            .unwrap();
        assert_eq!(response.status, 400);
        assert!(
            String::from_utf8(response.body)
                .unwrap()
                .contains("invalid logger. Use 'proxy', 'upstream', 'http' or a model's ID")
        );
        assert!(out.is_empty());
        let cut = copy_stream(&mut Trickle(UNKNOWN), &mut out, 10)
            .unwrap()
            .unwrap();
        assert_eq!(cut.body.len(), 10);
    }

    #[test]
    fn a_stream_cut_before_its_headers_is_an_error() {
        let mut out = Vec::new();
        assert_eq!(
            copy_stream(&mut Trickle(&STREAM[..30]), &mut out, 1024)
                .unwrap_err()
                .to_string(),
            "the response ends before its headers do"
        );
    }

    #[test]
    fn a_stream_whose_reader_has_gone_ends_with_the_write_error() {
        let error = copy_stream(&mut Trickle(STREAM), &mut ClosedPipe, 1024).unwrap_err();
        assert_eq!(error.to_string(), "cannot write the stream");
        assert!(broken_pipe(&error));
    }

    #[test]
    fn nothing_listening_is_not_an_error() {
        // A child forked while the port is bound would keep it listening.
        let _releasing = crate::infrastructure::command::exclusive();
        let addr = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        assert_eq!(
            request(addr, "GET", "/health", None, Duration::from_secs(2), None).unwrap(),
            None
        );
    }

    #[test]
    fn only_a_200_response_gives_its_body() {
        let timeout = Duration::from_secs(2);
        let (addr, _) = serve(HEALTH);
        assert_eq!(
            request_ok(addr, "GET", "/health", None, timeout, None).unwrap(),
            b"OK"
        );
        let (addr, _) = serve(NO_MODEL);
        assert_eq!(
            request_ok(addr, "GET", "/x", None, timeout, None)
                .unwrap_err()
                .to_string(),
            format!("GET http://{addr}/x answered 404: model not found")
        );
        // A child forked while the port is bound would keep it listening.
        let _releasing = crate::infrastructure::command::exclusive();
        let free = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let nothing = format!("nothing listens on {free}");
        let error = request_ok(free, "GET", "/x", None, timeout, None).unwrap_err();
        assert_eq!(error.to_string(), nothing);
        let error = open(free, "GET", "/logs", None, None, None).unwrap_err();
        assert_eq!(error.to_string(), nothing);
    }

    /// A server that takes one connection and never answers it.
    fn silent() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            // Until the client gives up and closes the connection.
            let _ = stream.read_to_end(&mut Vec::new());
        });
        addr
    }

    #[test]
    fn a_read_that_times_out_is_told_as_no_answer() {
        let addr = silent();
        let timeout = Duration::from_millis(50);
        let error = request(addr, "GET", "/running", None, timeout, None).unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            format!("GET http://{addr}/running: no answer within 50 ms")
        );

        let addr = silent();
        let timeout = Some(Duration::from_millis(50));
        let mut stream = open(addr, "POST", "/v1/chat/completions", None, timeout, None).unwrap();
        assert_eq!(
            format!("{:#}", read_head(&mut stream).unwrap_err()),
            "cannot read the response: no answer within 50 ms"
        );

        assert_eq!(duration(Duration::from_secs(300)), "300 s");
        assert_eq!(duration(Duration::from_millis(1500)), "1500 ms");
    }

    #[test]
    fn a_request_reaches_the_server_and_reads_its_answer() {
        let (addr, server) = serve(HEALTH);
        let response = request(addr, "GET", "/health", None, Duration::from_secs(2), None)
            .unwrap()
            .unwrap();
        assert_eq!(response.body, b"OK");
        assert_eq!(
            server.join().unwrap(),
            format!(
                "GET /health HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
            )
        );
    }

    #[test]
    fn a_key_goes_in_a_bearer_authorization() {
        let (addr, server) = serve(HEALTH);
        let timeout = Duration::from_secs(2);
        request(addr, "GET", "/running", None, timeout, Some("k-1")).unwrap();
        assert_eq!(
            server.join().unwrap(),
            format!(
                "GET /running HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\nAuthorization: Bearer k-1\r\nContent-Length: 0\r\n\r\n"
            )
        );
    }

    #[test]
    fn a_json_body_is_sent_with_its_type_and_length() {
        let (addr, server) = serve(HEALTH);
        let mut stream = send(addr, "POST", "/v1/x", Some(b"{\"a\":1}"), None, None)
            .unwrap()
            .unwrap();
        stream.read_to_end(&mut Vec::new()).unwrap();
        assert_eq!(
            server.join().unwrap(),
            format!(
                "POST /v1/x HTTP/1.0\r\nHost: {addr}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: 7\r\n\r\n{{\"a\":1}}"
            )
        );
    }
}
