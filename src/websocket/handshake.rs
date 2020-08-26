use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use regex::Regex;

use super::parsed_addr::ParsedAddr;
use super::WebSocket;
use crate::error::WebSocketError;

// length of "HTTP/1.1 101 Switching Protocols"
const PROBABLE_STATUS_LINE_LENGTH: usize = 34;

pub(super) struct Handshake {
    path: String,
    host: String,
    key: String,
    version: usize,
    additional_headers: Vec<(String, String)>,
    subprotocols: Vec<String>,
}

impl Handshake {
    pub(super) fn new(
        parsed_addr: &ParsedAddr,
        additional_handshake_headers: &Vec<(String, String)>,
        subprotocols: &Vec<String>,
    ) -> Result<Self, WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.3
        let mut rand_bytes = Vec::with_capacity(16);
        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut rand_bytes);
        let key = base64::encode(rand_bytes);
        Ok(Self {
            path: parsed_addr.path.clone(),
            host: parsed_addr.host.clone(),
            key,
            // todo: support more versions
            version: 13,
            additional_headers: additional_handshake_headers.clone(),
            subprotocols: subprotocols.clone(),
        })
    }

    pub(super) async fn send_request(&self, ws: &mut WebSocket) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-1.3
        // https://tools.ietf.org/html/rfc6455#section-4.1
        let mut headers = Vec::new();
        headers.push(("Host".into(), self.host.clone()));
        headers.push(("Upgrade".into(), "websocket".into()));
        headers.push(("Connection".into(), "Upgrade".into()));
        headers.push(("Sec-WebSocket-Key".into(), self.key.clone()));
        headers.push(("Sec-Websocket-Version".into(), self.version.to_string()));
        if self.subprotocols.len() > 0 {
            headers.push((
                "Sec-WebSocket-Protocol".into(),
                self.subprotocols.join(", "),
            ));
        }
        for header in &self.additional_headers {
            headers.push(header.clone());
        }

        let mut req = format!("GET {} HTTP/1.1\r\n", self.path);
        for (field, value) in headers {
            req.push_str(&format!("{}: {}\r\n", field, value));
        }
        req.push_str("\r\n"); // end of request
        ws.stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        Ok(())
    }

    pub(super) async fn check_response(&self, ws: &mut WebSocket) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-1.3
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        // sec-websocket-accept
        // sec-websocket-version
        // sec-websocket-protocol
        let status_line_regex = Regex::new(r"HTTP/\d+\.\d+ (\d{3}) (.+)\r\n").unwrap();
        let mut status_line = String::with_capacity(PROBABLE_STATUS_LINE_LENGTH);
        ws.stream
            .read_line(&mut status_line)
            .await
            .map_err(|e| WebSocketError::ReadError(e))?;
        let captures = status_line_regex.captures(&status_line).ok_or(WebSocketError::InvalidHandshakeError)?;
        let status_code = &captures[1];
        // let reason_phrase = captures.get(2).ok_or(WebSocketError::InvalidHandshakeError);
        

        // set accepted subprotocols
        // set handshake response headers
        unimplemented!()
    }
}
