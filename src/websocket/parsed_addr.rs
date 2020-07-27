use std::net::SocketAddr;

use super::scheme::Scheme;

#[derive(Debug)]
pub(crate) struct ParsedAddr {
    pub scheme: Scheme,
    pub host: String,
    pub addr: SocketAddr,
    pub url: String,
}
