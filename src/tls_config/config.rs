use native_tls::{TlsConnector, TlsConnectorBuilder};

use super::certificate::Certificate;
use super::identity::Identity;
use super::protocol::Protocol;
use crate::error::WebSocketError;

pub struct TlsConfig(TlsConnectorBuilder);

impl TlsConfig {
    pub fn new() -> Self {
        Self(TlsConnector::builder())
    }

    pub fn identity(mut self, identity: Identity) -> Self {
        self.0.identity(identity.into());
        self
    }

    pub fn min_protocol_version(mut self, protocol: Option<Protocol>) -> Self {
        self.0.min_protocol_version(protocol.map(|p| p.into()));
        self
    }

    pub fn max_protocol_version(mut self, protocol: Option<Protocol>) -> Self {
        self.0.max_protocol_version(protocol.map(|p| p.into()));
        self
    }

    pub fn add_root_certificate(mut self, cert: Certificate) -> Self {
        self.0.add_root_certificate(cert.into());
        self
    }

    pub fn danger_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.0.danger_accept_invalid_certs(accept_invalid_certs);
        self
    }

    pub fn use_sni(mut self, use_sni: bool) -> Self {
        self.0.use_sni(use_sni);
        self
    }

    pub fn danger_accept_invalid_hostnames(mut self, accept_invalid_hostnames: bool) -> Self {
        self.0
            .danger_accept_invalid_hostnames(accept_invalid_hostnames);
        self
    }

    pub fn build(self) -> Result<TlsConnector, WebSocketError> {
        self.0
            .build()
            .map_err(|e| WebSocketError::TlsConfigError(e))
    }
}
