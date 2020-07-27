use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite};

use super::builder::WebSocketBuilder;
use super::parsed_addr::ParsedAddr;
use super::stream::Stream;
use crate::error::WebSocketError;

#[derive(Debug)]
pub struct WebSocket {
    pub(crate) parsed_addr: ParsedAddr,
    pub(crate) stream: Stream,
}

impl AsyncRead for WebSocket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, IoError>> {
        unimplemented!();
    }
}

impl AsyncWrite for WebSocket {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        unimplemented!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        unimplemented!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        unimplemented!()
    }
}

impl WebSocket {
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::new()
    }

    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        WebSocketBuilder::new().connect(url).await
    }

    fn close() {
        unimplemented!()
    }
}
