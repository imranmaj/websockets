use std::convert::TryFrom;

use tokio::io::BufStream;
use tokio::net::TcpStream;

use super::handshake::Handshake;
use super::parsed_addr::ParsedAddr;
use super::stream::Stream;
use super::FrameType;
use super::WebSocket;
use crate::error::WebSocketError;

#[derive(Debug)]
pub struct WebSocketBuilder {
    additional_handshake_headers: Vec<(String, String)>,
    subprotocols: Vec<String>,
}

impl WebSocketBuilder {
    pub(super) fn new() -> Self {
        Self {
            additional_handshake_headers: Vec::new(),
            subprotocols: Vec::new(),
        }
    }

    pub fn add_header(&mut self, header_name: &str, header_value: &str) -> &mut Self {
        self.additional_handshake_headers
            .push((header_name.to_string(), header_value.to_string()));
        self
    }

    pub fn remove_header(&mut self, header_name: &str) -> &mut Self {
        self.additional_handshake_headers
            .retain(|header| header.0 != header_name);
        self
    }

    pub fn add_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        self.subprotocols.push(subprotocol.to_string());
        self
    }

    pub fn remove_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        self.subprotocols.retain(|s| s != subprotocol);
        self
    }

    pub async fn connect(&mut self, url: &str) -> Result<WebSocket, WebSocketError> {
        let parsed_addr = ParsedAddr::try_from(url)?;

        let stream = Stream::Plain(
            TcpStream::connect(parsed_addr.addr)
                .await
                .map_err(|e| WebSocketError::TcpConnectionError(e))?,
        );
        let stream = match &parsed_addr.scheme[..] {
            "ws" => stream,
            "wss" => {
                stream
                    .into_tls(&parsed_addr.host)
                    .await?
            }
            _ => return Err(WebSocketError::SchemeError),
        };
        let mut ws = WebSocket {
            stream: BufStream::new(stream),
            shutdown: false,
            last_frame_type: FrameType::Control,
            accepted_subprotocol: None,
            handshake_response_headers: None,
        };

        // perform opening handshake
        let handshake = Handshake::new(
            &parsed_addr,
            &self.additional_handshake_headers,
            &self.subprotocols,
        )?;
        handshake.send_request(&mut ws).await?;
        handshake.check_response(&mut ws).await?;
        Ok(ws)
    }
}
