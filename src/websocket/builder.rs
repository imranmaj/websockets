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

    pub fn tls_config(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(tls_config);
        self
    }

    pub fn handshake_config(mut self, handshake_config: HandshakeConfig) -> Self {
        self.handshake_config = Some(handshake_config);
        self
    }

    pub async fn connect(self, addr: &str) -> Result<WebSocket, WebSocketError> {
        let tls_config = self.tls_config.unwrap_or(TlsConfig::new());
        let handshake_config = self.handshake_config.unwrap_or(HandshakeConfig::new());
        WebSocket::connect(addr, tls_config, handshake_config).await
    }
}
