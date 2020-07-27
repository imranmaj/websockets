use std::convert::TryInto;
use std::net::ToSocketAddrs;

use tokio::net::TcpStream;
use url::Url;

use super::parsed_addr::ParsedAddr;
use super::scheme::Scheme;
use super::stream::Stream;
use super::websocket::WebSocket;
use crate::error::WebSocketError;
use crate::handshake_config::HandshakeConfig;
use crate::tls_config::TlsConfig;

pub struct WebSocketBuilder {
    tls_config: Option<TlsConfig>,
    handshake_config: Option<HandshakeConfig>,
}

impl WebSocketBuilder {
    pub fn new() -> Self {
        Self {
            tls_config: None,
            handshake_config: None,
        }
    }

    pub fn tls_config(&mut self, tls_config: TlsConfig) -> &mut Self {
        self.tls_config = Some(tls_config);
        self
    }

    pub fn handshake_config(&mut self, handshake_config: HandshakeConfig) -> &mut Self {
        self.handshake_config = Some(handshake_config);
        self
    }

    pub async fn connect(&mut self, url: &str) -> Result<WebSocket, WebSocketError> {
        let tls_config = self.tls_config.take().unwrap_or(TlsConfig::new());
        let handshake_config = self.handshake_config.take().unwrap_or(HandshakeConfig::new());
        let parsed_addr = parse_url(url)?;
        let stream = Stream::Plain(
            TcpStream::connect(parsed_addr.addr)
                .await
                .map_err(|e| WebSocketError::TcpConnectionError(e))?,
        );
        let stream = match parsed_addr.scheme {
            Scheme::Plain => stream,
            Scheme::Secure => stream.into_tls(&parsed_addr.host, tls_config).await?,
        };
        let ws = WebSocket {
            parsed_addr,
            stream,
        };
        handshake(ws, handshake_config).await
    }
}

async fn handshake(
    ws: WebSocket,
    handshake_config: HandshakeConfig,
) -> Result<WebSocket, WebSocketError> {
    unimplemented!()
}

fn parse_url(url: &str) -> Result<ParsedAddr, WebSocketError> {
    let parsed_url = Url::parse(url).map_err(|e| WebSocketError::ParseError(e))?;
    let scheme = parsed_url.scheme().try_into()?;
    let host = parsed_url.host_str().ok_or(WebSocketError::HostError)?;
    let port = parsed_url
        .port_or_known_default()
        .ok_or(WebSocketError::PortError)?;
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| WebSocketError::SocketAddrError(e))?
        .next()
        .ok_or(WebSocketError::ResolutionError)?;
    Ok(ParsedAddr {
        scheme,
        host: host.into(),
        addr,
        url: url.into(),
    })
}
