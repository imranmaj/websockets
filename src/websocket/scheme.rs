use std::convert::TryFrom;

use crate::error::WebSocketError;

#[derive(Debug)]
pub(crate) enum Scheme {
    Plain,
    Secure,
}

impl TryFrom<&str> for Scheme {
    type Error = WebSocketError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ws" => Ok(Self::Plain),
            "wss" => Ok(Self::Secure),
            _ => Err(WebSocketError::SchemeError),
        }
    }
}