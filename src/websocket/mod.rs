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
enum DataFrameType {
    Text,
    Binary,
    Control,
}

#[derive(Debug)]
pub struct WebSocket {
    stream: Stream,
    closed: bool,
    last_data_frame_type: DataFrameType,
    accepted_subprotocols: Option<Vec<String>>,
}

impl WebSocket {
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::new()
    }

    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        WebSocketBuilder::new().connect(url).await
    }

    pub fn accepted_subprotocols(&self) -> &Option<Vec<String>> {
        &self.accepted_subprotocols
    }

    pub async fn send(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        if self.closed {
            return Err(WebSocketError::WebSocketClosedError);
        }
        let raw_frame = frame.into_raw()?;
        self.stream
            .write_all(&raw_frame)
            .await
            .map_err(|e| WebSocketError::WriteError(e))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Frame, WebSocketError> {
        if self.closed {
            return Err(WebSocketError::WebSocketClosedError);
        }
        let frame = Frame::read_from_websocket(self).await?;
        // remember last data frame type in case we get continuation frames
        match frame {
            Frame::Text { .. } => self.last_data_frame_type = DataFrameType::Text,
            Frame::Binary { .. } => self.last_data_frame_type = DataFrameType::Binary,
            _ => (),
        };
        // handle incoming frames
        match &frame {
            // echo ping frame
            Frame::Ping { payload } => {
                let pong = Frame::Pong {
                    payload: payload.clone(),
                };
                self.send(pong).await?;
            }
            // echo close frame and close
            Frame::Close { payload } => {
                let close = Frame::Close {
                    payload: payload
                        .as_ref()
                        .map(|(status_code, _reason)| (status_code.clone(), String::new())),
                };
                self.send(close).await?;
                self.close().await?;
            }
            _ => (),
        }
        Ok(frame)
    }

    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        if self.closed {
            Err(WebSocketError::WebSocketClosedError)
        } else {
            self.send(Frame::Close { payload: None }).await?;
            self.closed = true;
            self.stream
                .shutdown()
                .await
                .map_err(|e| WebSocketError::ShutdownError(e))?;
            Ok(())
        }
    }
}
