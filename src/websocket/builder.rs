use std::convert::TryFrom;

use tokio::io::BufStream;
use tokio::net::TcpStream;

use super::handshake::Handshake;
use super::parsed_addr::ParsedAddr;
use super::stream::Stream;
use super::FrameType;
use super::WebSocket;
use crate::error::WebSocketError;

/// A builder used to customize the WebSocket handshake.
/// ```
/// # use websockets::WebSocket;
/// # #[tokio::main]
/// # async fn main() {
/// let mut ws = WebSocket::builder()
///     .add_subprotocol("wamp")
///     .connect("wss://echo.websocket.org")
///     .await
///     .unwrap();
/// # }
/// ```
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

    /// Builds a [`WebSocket`] using this builder, then connects to a URL 
    /// (and performs the WebSocket handshake).
    /// 
    /// After calling this method, no more methods should be called on this builder.
    pub async fn connect(&mut self, url: &str) -> Result<WebSocket, WebSocketError> {
        let parsed_addr = ParsedAddr::try_from(url)?;

        let stream = Stream::Plain(
            TcpStream::connect(parsed_addr.addr)
                .await
                .map_err(|e| WebSocketError::TcpConnectionError(e))?,
        );
        let stream = match &parsed_addr.scheme[..] {
            // https://tools.ietf.org/html/rfc6455#section-11.1.1
            "ws" => stream,
            // https://tools.ietf.org/html/rfc6455#section-11.1.2
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
        );
        handshake.send_request(&mut ws).await?;
        match handshake.check_response(&mut ws).await {
            Ok(_) => Ok(ws),
            Err(e) => {
                ws.shutdown().await?;
                Err(e)
            }
        }
    }

    /// Adds a header to be sent in the WebSocket handshake.
    pub fn add_header(&mut self, header_name: &str, header_value: &str) -> &mut Self {
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        self.additional_handshake_headers
            .push((header_name.to_string(), header_value.to_string()));
        self
    }

    /// Removes a header which would be sent in the WebSocket handshake.
    pub fn remove_header(&mut self, header_name: &str) -> &mut Self {
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        self.additional_handshake_headers
            .retain(|header| header.0 != header_name);
        self
    }

    /// Adds a subprotocol to the list of subprotocols to be sent in the
    /// WebSocket handshake. The server may select a subprotocol from this list.
    /// If it does, the selected subprotocol can be found using the
    /// [`WebSocket::accepted_subprotocol()`] method.
    pub fn add_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        // https://tools.ietf.org/html/rfc6455#section-1.9
        self.subprotocols.push(subprotocol.to_string());
        self
    }

    /// Removes a subprotocol from the list of subprotocols that would be sent
    /// in the WebSocket handshake. 
    pub fn remove_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        // https://tools.ietf.org/html/rfc6455#section-1.9
        self.subprotocols.retain(|s| s != subprotocol);
        self
    }
}
