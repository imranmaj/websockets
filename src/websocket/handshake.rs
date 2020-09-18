use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use regex::Regex;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

use super::parsed_addr::ParsedAddr;
use super::WebSocket;
use crate::error::WebSocketError;

const GUUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
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
    ) -> Self {
        // https://tools.ietf.org/html/rfc6455#section-5.3
        let mut rand_bytes = vec![0; 16];
        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut rand_bytes);
        let key = base64::encode(rand_bytes);
        Self {
            path: parsed_addr.path.clone(),
            host: parsed_addr.host.clone(),
            key,
            // todo: support more versions
            version: 13,
            additional_headers: additional_handshake_headers.clone(),
            subprotocols: subprotocols.clone(),
        }
    }

    pub(super) async fn send_request(&self, ws: &mut WebSocket) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-1.3
        // https://tools.ietf.org/html/rfc6455#section-4.1
        let mut headers = Vec::new();
        headers.push(("Host".to_string(), self.host.clone()));
        headers.push(("Upgrade".to_string(), "websocket".to_string()));
        headers.push(("Connection".to_string(), "Upgrade".to_string()));
        headers.push(("Sec-WebSocket-Key".to_string(), self.key.clone()));
        headers.push((
            "Sec-Websocket-Version".to_string(),
            self.version.to_string(),
        ));
        if self.subprotocols.len() > 0 {
            headers.push((
                "Sec-WebSocket-Protocol".to_string(),
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
        ws.write_half
            .stream
            .write_all(req.as_bytes())
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        ws.write_half
            .stream
            .flush()
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        Ok(())
    }

    pub(super) async fn check_response(&self, ws: &mut WebSocket) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-1.3
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        let status_line_regex = Regex::new(r"HTTP/\d+\.\d+ (?P<status_code>\d{3}) .+\r\n").unwrap();
        let mut status_line = String::new();

        ws.read_half
            .stream
            .read_line(&mut status_line)
            .await
            .map_err(|e| WebSocketError::ReadError(e))?;
        let captures = status_line_regex
            .captures(&status_line)
            .ok_or(WebSocketError::InvalidHandshakeError)?;
        let status_code = &captures["status_code"];

        let mut headers = Vec::new();
        let headers_regex = Regex::new(r"(?P<field>.+?):\s*(?P<value>.*?)\s*\r\n").unwrap();
        loop {
            let mut header = String::new();
            ws.read_half
                .stream
                .read_line(&mut header)
                .await
                .map_err(|e| WebSocketError::ReadError(e))?;
            match headers_regex.captures(&header) {
                Some(captures) => {
                    let field = &captures["field"];
                    let value = &captures["value"];
                    headers.push((field.to_string(), value.to_string()));
                }
                None => break, // field is empty, so the header is finished (we got double crlf)
            }
        }

        // check status code
        if status_code != "101" {
            let body = match headers
                .iter()
                .find(|(field, _value)| field.to_lowercase() == "content-length")
            {
                Some(header) => {
                    let body_length = header
                        .1
                        .parse::<usize>()
                        .map_err(|_e| WebSocketError::InvalidHandshakeError)?;
                    let mut body = vec![0; body_length];
                    ws.read_half
                        .stream
                        .read_exact(&mut body)
                        .await
                        .map_err(|e| WebSocketError::ReadError(e))?;
                    Some(
                        String::from_utf8(body)
                            .map_err(|_e| WebSocketError::InvalidHandshakeError)?,
                    )
                }
                None => None,
            };
            return Err(WebSocketError::HandshakeFailedError {
                status_code: status_code.to_string(),
                headers,
                body,
            });
        }

        // check upgrade field
        let upgrade = headers
            .iter()
            .find(|(field, _value)| field.to_lowercase() == "upgrade")
            .ok_or(WebSocketError::InvalidHandshakeError)?
            .1
            .clone();
        if upgrade.to_lowercase() != "websocket" {
            return Err(WebSocketError::InvalidHandshakeError);
        }

        // check connection field
        let connection = headers
            .iter()
            .find(|(field, _value)| field.to_lowercase() == "connection")
            .ok_or(WebSocketError::InvalidHandshakeError)?
            .1
            .clone();
        if connection.to_lowercase() != "upgrade" {
            return Err(WebSocketError::InvalidHandshakeError);
        }

        // check extensions
        if let Some(_) = headers
            .iter()
            .find(|(field, _value)| field.to_lowercase() == "sec-websocket-extensions")
        {
            // extensions not supported
            return Err(WebSocketError::InvalidHandshakeError);
        }

        // check subprotocols
        let possible_subprotocol = headers
            .iter()
            .find(|(field, _value)| field.to_lowercase() == "sec-websocket-protocol")
            .map(|(_field, value)| value.clone());
        match (possible_subprotocol, self.subprotocols.len()) {
            // server accepted a subprotocol that was not specified
            (Some(_), 0) => return Err(WebSocketError::InvalidHandshakeError),
            // server accepted a subprotocol that may have been specified
            (Some(subprotocol), _) => {
                if self.subprotocols.contains(&subprotocol) {
                    ws.accepted_subprotocol = Some(subprotocol)
                } else {
                    return Err(WebSocketError::InvalidHandshakeError);
                }
            }
            // server did not accept a subprotocol, whether one was specified or not
            (None, _) => (),
        }

        // validate key
        let accept_key = headers
            .iter()
            .find(|(field, _value)| field.to_lowercase() == "sec-websocket-accept")
            .ok_or(WebSocketError::InvalidHandshakeError)?
            .1
            .clone();
        let mut test_key = self.key.clone();
        test_key.push_str(GUUID);
        let hashed: [u8; 20] = Sha1::digest(test_key.as_bytes()).into();
        let calculated_accept_key = base64::encode(hashed);
        if accept_key != calculated_accept_key {
            return Err(WebSocketError::InvalidHandshakeError);
        }

        ws.handshake_response_headers = Some(headers);
        Ok(())
    }
}
