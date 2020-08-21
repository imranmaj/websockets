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
    None,
}

#[derive(Debug)]
pub struct WebSocket {
    stream: Stream,
    closed: bool,
    last_data_frame_type: DataFrameType,
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
        let frame = Frame::read_from_stream(&mut self.stream, &self.last_data_frame_type).await?;
        // remember last data frame type in case we get continuation frames
        self.last_data_frame_type = match frame {
            Frame::Text { .. } => DataFrameType::Text,
            Frame::Binary { .. } => DataFrameType::Binary,
            _ => self.last_data_frame_type,
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
        if !self.closed {
            self.send(Frame::Close { payload: None }).await?;
            self.closed = true;
            self.stream
                .shutdown()
                .await
                .map_err(|e| WebSocketError::ShutdownError(e))?;
        }
        Ok(())
    }
}
