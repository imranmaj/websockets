mod certificate;
mod identity;
mod protocol;

use std::convert::TryFrom;

// use rand::{RngCore, SeedableRng};
// use rand_chacha::ChaCha20Rng;
use native_tls::{TlsConnector, TlsConnectorBuilder};
use tokio::io::BufStream;
use tokio::net::TcpStream;

use super::handshake::Handshake;
use super::parsed_addr::ParsedAddr;
use super::stream::Stream;
use super::FrameType;
use super::WebSocket;
use crate::error::WebSocketError;
pub use certificate::Certificate;
pub use identity::Identity;
pub use protocol::Protocol;
// use crate::handshake_config::HandshakeConfig;
// use crate::tls_config::TlsConfig;

pub struct WebSocketBuilder {
    // tls_config: Option<TlsConfig>,
    tls_builder: TlsConnectorBuilder,
    // handshake_config: Option<HandshakeConfig>,
    additional_handshake_headers: Vec<(String, String)>,
    subprotocols: Vec<String>,
}

impl WebSocketBuilder {
    pub(super) fn new() -> Self {
        Self {
            tls_builder: TlsConnector::builder(),
            additional_handshake_headers: Vec::new(),
            subprotocols: Vec::new(),
        }
    }

    pub fn handshake_add_header(&mut self, header_name: &str, header_value: &str) -> &mut Self {
        self.additional_handshake_headers
            .push((header_name.into(), header_value.into()));
        self
    }

    pub fn handshake_remove_header(&mut self, header_name: &str) -> &mut Self {
        self.additional_handshake_headers
            .retain(|header| header.0 != header_name);
        self
    }

    pub fn handshake_add_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        self.subprotocols.push(subprotocol.into());
        self
    }

    pub fn handshake_remove_subprotocol(&mut self, subprotocol: &str) -> &mut Self {
        self.subprotocols.retain(|s| s != subprotocol);
        self
    }

    pub fn tls_identity(&mut self, identity: Identity) -> &mut Self {
        self.tls_builder.identity(identity.into());
        self
    }

    pub fn tls_min_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut Self {
        self.tls_builder
            .min_protocol_version(protocol.map(|p| p.into()));
        self
    }

    pub fn tls_max_protocol_version(&mut self, protocol: Option<Protocol>) -> &mut Self {
        self.tls_builder
            .max_protocol_version(protocol.map(|p| p.into()));
        self
    }

    pub fn tls_add_root_certificate(&mut self, cert: Certificate) -> &mut Self {
        self.tls_builder.add_root_certificate(cert.into());
        self
    }

    pub fn tls_danger_accept_invalid_certs(&mut self, accept_invalid_certs: bool) -> &mut Self {
        self.tls_builder
            .danger_accept_invalid_certs(accept_invalid_certs);
        self
    }

    pub fn tls_use_sni(&mut self, use_sni: bool) -> &mut Self {
        self.tls_builder.use_sni(use_sni);
        self
    }

    pub fn tls_danger_accept_invalid_hostnames(
        &mut self,
        accept_invalid_hostnames: bool,
    ) -> &mut Self {
        self.tls_builder
            .danger_accept_invalid_hostnames(accept_invalid_hostnames);
        self
    }

    // pub fn tls_config(&mut self, tls_config: TlsConfig) -> &mut Self {
    //     self.tls_config = Some(tls_config);
    //     self
    // }

    // pub fn handshake_config(&mut self, handshake_config: HandshakeConfig) -> &mut Self {
    //     self.handshake_config = Some(handshake_config);
    //     self
    // }

    pub async fn connect(&mut self, url: &str) -> Result<WebSocket, WebSocketError> {
        // let tls_config = self.tls_config.take().unwrap_or(TlsConfig::new());
        // let handshake_config = self
        //     .handshake_config
        //     .take()
        //     .unwrap_or(HandshakeConfig::new());
        let parsed_addr = ParsedAddr::try_from(url)?;

        // let ws = connect(&parsed_addr, tls_config).await?;
        let stream = Stream::Plain(
            TcpStream::connect(parsed_addr.addr)
                .await
                .map_err(|e| WebSocketError::TcpConnectionError(e))?,
        );
        let stream = match &*parsed_addr.scheme {
            "ws" => stream,
            "wss" => {
                stream
                    .into_tls(&parsed_addr.host, &self.tls_builder)
                    .await?
            }
            _ => return Err(WebSocketError::SchemeError),
        };
        let mut ws = WebSocket {
            stream: BufStream::new(stream),
            closed: false,
            last_frame_type: FrameType::Control,
            accepted_subprotocols: None,
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

// async fn connect(
//     parsed_addr: &ParsedAddr,
//     tls_config: TlsConfig,
// ) -> Result<WebSocket, WebSocketError> {
//     let stream = Stream::Plain(
//         TcpStream::connect(parsed_addr.addr)
//             .await
//             .map_err(|e| WebSocketError::TcpConnectionError(e))?,
//     );
//     let stream = match &parsed_addr.scheme[..] {
//         "ws" => stream,
//         "wss" => stream.into_tls(&parsed_addr.host, tls_config).await?,
//         _ => return Err(WebSocketError::SchemeError),
//     };
//     Ok(WebSocket { stream })
// }

// async fn handshake(
//     mut ws: WebSocket,
//     parsed_addr: &ParsedAddr,
//     handshake_config: HandshakeConfig,
// ) -> Result<WebSocket, WebSocketError> {
//     let handshake = Handshake::new(parsed_addr, handshake_config)?;
//     // unimplemented!();
//     ws.stream.write_all(&handshake.make_request()).await;
//     let mut resp = Vec::new();
//     ws.stream
//         .read_to_end(&mut resp)
//         .await
//         .map_err(|e| WebSocketError::ReadError(e))?;
//     handshake.check_response(&resp).map(|_e| ws)
// }
