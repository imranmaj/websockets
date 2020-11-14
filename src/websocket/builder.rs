use std::convert::TryFrom;
use std::sync::mpsc;

use native_tls::{
    TlsConnector as NativeTlsConnector, TlsConnectorBuilder as NativeTlsConnectorBuilder,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use tokio::io::{self, BufReader, BufWriter};
use tokio::net::TcpStream;

use super::handshake::Handshake;
use super::parsed_addr::ParsedAddr;
use super::split::{WebSocketReadHalf, WebSocketWriteHalf};
use super::stream::Stream;
use super::FrameType;
use super::WebSocket;
use crate::error::WebSocketError;
use crate::secure::{TlsCertificate, TlsIdentity, TlsProtocol};

/// A builder used to customize the WebSocket handshake.
/// ```
/// # use websockets::{WebSocket, WebSocketError};
/// # #[tokio::main]
/// # async fn main() -> Result<(), WebSocketError> {
/// let mut ws = WebSocket::builder()
///     .add_subprotocol("wamp")
///     .connect("wss://echo.websocket.org")
///     .await?;
/// # Ok(())
/// # }
/// ```
#[allow(missing_debug_implementations)]
pub struct WebSocketBuilder {
    additional_handshake_headers: Vec<(String, String)>,
    subprotocols: Vec<String>,
    tls_connector_builder: NativeTlsConnectorBuilder,
}

impl WebSocketBuilder {
    pub(super) fn new() -> Self {
        Self {
            additional_handshake_headers: Vec::new(),
            subprotocols: Vec::new(),
            tls_connector_builder: NativeTlsConnector::builder(),
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
                let ssl_config = self
                    .tls_connector_builder
                    .build()
                    .map_err(|e| WebSocketError::TlsConnectionError(e))?;
                stream.into_tls(&parsed_addr.host, ssl_config).await?
            }
            _ => return Err(WebSocketError::SchemeError),
        };
        let (read_half, write_half) = io::split(stream);
        let (sender, receiver) = mpsc::channel();
        let mut ws = WebSocket {
            read_half: WebSocketReadHalf {
                stream: BufReader::new(read_half),
                last_frame_type: FrameType::default(),
                sender,
            },
            write_half: WebSocketWriteHalf {
                shutdown: false,
                sent_closed: false,
                stream: BufWriter::new(write_half),
                rng: ChaCha20Rng::from_entropy(),
                receiver,
            },
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

    /// Controls the use of certificate validation. Defaults to false.
    pub fn danger_accept_invalid_certs(&mut self, accept_invalid_certs: bool) -> &mut Self {
        self.tls_connector_builder
            .danger_accept_invalid_certs(accept_invalid_certs);
        self
    }

    /// Controls the use of hostname verification. Defaults to false.
    pub fn danger_accept_invalid_hostnames(&mut self, accept_invalid_hostnames: bool) -> &mut Self {
        self.tls_connector_builder
            .danger_accept_invalid_hostnames(accept_invalid_hostnames);
        self
    }

    /// Adds a certificate to the set of roots that the connector will trust.
    /// The connector will use the system's trust root by default. This method can be used to add
    /// to that set when communicating with servers not trusted by the system.
    /// Defaults to an empty set.
    pub fn add_root_certificate(&mut self, cert: TlsCertificate) -> &mut Self {
        self.tls_connector_builder.add_root_certificate(cert);
        self
    }

    /// Controls the use of built-in system certificates during certificate validation.
    /// Defaults to false -- built-in system certs will be used.
    pub fn disable_built_in_roots(&mut self, disable: bool) -> &mut Self {
        self.tls_connector_builder.disable_built_in_roots(disable);
        self
    }

    /// Sets the identity to be used for client certificate authentication.
    pub fn identity(&mut self, identity: TlsIdentity) -> &mut Self {
        self.tls_connector_builder.identity(identity);
        self
    }

    /// Sets the maximum supported TLS protocol version.
    /// A value of None enables support for the newest protocols supported by the implementation.
    /// Defaults to None.
    pub fn max_protocol_version(&mut self, protocol: Option<TlsProtocol>) -> &mut Self {
        self.tls_connector_builder.max_protocol_version(protocol);
        self
    }

    /// Sets the minimum supported TLS protocol version.
    /// A value of None enables support for the oldest protocols supported by the implementation.
    /// Defaults to Some(Protocol::Tlsv10).
    pub fn min_protocol_version(&mut self, protocol: Option<TlsProtocol>) -> &mut Self {
        self.tls_connector_builder.min_protocol_version(protocol);
        self
    }

    /// Controls the use of Server Name Indication (SNI).
    /// Defaults to true.
    pub fn use_sni(&mut self, use_sni: bool) -> &mut Self {
        self.tls_connector_builder.use_sni(use_sni);
        self
    }
}
