pub mod builder;
pub mod frame;
mod handshake;
mod parsed_addr;
mod stream;

use tokio::io::AsyncWriteExt;

use crate::error::WebSocketError;
use builder::WebSocketBuilder;
use frame::Frame;
use stream::Stream;

#[derive(Debug)]
pub struct WebSocket {
    stream: Stream,
    closed: bool,
}

impl WebSocket {
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::new()
    }

    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        WebSocketBuilder::new().connect(url).await
    }

    pub async fn send(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        if self.closed {
            return Err(WebSocketError::WebSocketClosedError);
        }
        let raw_frame = frame.into_raw();
        unimplemented!()
    }

    pub async fn receive(&mut self) -> Result<Frame, WebSocketError> {
        if self.closed {
            return Err(WebSocketError::WebSocketClosedError);
        }
        unimplemented!()
        // handle ping frame (send pong)
        // handle close frame (echo close, set self.closed)
    }

    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        if !self.closed {
            self.send(Frame::Close{payload: None}).await?;
            self.closed = true;
        }
        self.stream.shutdown().await.map_err(|e| WebSocketError::ShutdownError(e))?;
        Ok(())
    }
}
