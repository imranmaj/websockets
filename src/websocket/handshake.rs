use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use super::parsed_addr::ParsedAddr;
use crate::error::WebSocketError;

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
    ) -> Result<Self, WebSocketError> {
        let mut rand_bytes = Vec::with_capacity(16);
        let mut rng = ChaCha20Rng::from_entropy();
        rng.fill_bytes(&mut rand_bytes);
        let key = base64::encode(rand_bytes);
        Ok(Self {
            path: parsed_addr.path.clone(),
            host: parsed_addr.host.clone(),
            key,
            // todo: support more versions
            version: 13,
            additional_headers: additional_handshake_headers.clone(),
            subprotocols: subprotocols.clone(),
        })
    }

    pub(super) fn make_request(&self) -> Vec<u8> {
        unimplemented!()
    }

    pub(super) fn check_response(&self, resp: &[u8]) -> Result<(), WebSocketError> {
        unimplemented!()
    }
}
