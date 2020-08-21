use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use url::Url;

use crate::WebSocketError;

#[derive(Debug)]
pub(super) struct ParsedAddr {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub addr: SocketAddr,
}

impl TryFrom<&str> for ParsedAddr {
    type Error = WebSocketError;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        let parsed_url = Url::parse(url).map_err(|e| WebSocketError::ParseError(e))?;
        let scheme = parsed_url.scheme();
        let host = parsed_url.host_str().ok_or(WebSocketError::HostError)?;
        let path = parsed_url.path();
        let port = parsed_url
            .port_or_known_default()
            .ok_or(WebSocketError::PortError)?;
        let addr = (host, port)
            .to_socket_addrs()
            .map_err(|e| WebSocketError::SocketAddrError(e))?
            .next()
            .ok_or(WebSocketError::ResolutionError)?;
        Ok(ParsedAddr {
            scheme: scheme.into(),
            host: host.into(),
            path: path.into(),
            addr,
        })
    }
}
