pub mod builder;
pub mod frame;
mod handshake;
mod parsed_addr;
mod stream;

use tokio::io::{AsyncWriteExt, BufStream};

use crate::error::WebSocketError;
use builder::WebSocketBuilder;
use frame::Frame;
use stream::Stream;

#[derive(Debug)]
enum FrameType {
    Text,
    Binary,
    Control,
}

#[derive(Debug)]
pub struct WebSocket {
    stream: BufStream<Stream>,
    shutdown: bool,
    last_frame_type: FrameType,
    accepted_subprotocol: Option<String>,
    handshake_response_headers: Option<Vec<(String, String)>>,
}

impl WebSocket {
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::new()
    }

    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        WebSocketBuilder::new().connect(url).await
    }

    pub fn accepted_subprotocol(&self) -> &Option<String> {
        &self.accepted_subprotocol
    }

    pub fn handshake_response_headers(&self) -> &Option<Vec<(String, String)>> {
        &self.handshake_response_headers
    }

    pub async fn send(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        if self.shutdown {
            return Err(WebSocketError::WebSocketClosedError);
        }
        frame.send(self).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Frame, WebSocketError> {
        if self.shutdown {
            return Err(WebSocketError::WebSocketClosedError);
        }
        let frame = Frame::read_from_websocket(self).await?;
        // remember last data frame type in case we get continuation frames
        match frame {
            Frame::Text { .. } => self.last_frame_type = FrameType::Text,
            Frame::Binary { .. } => self.last_frame_type = FrameType::Binary,
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
            // echo close frame and close (https://tools.ietf.org/html/rfc6455#section-1.4)
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
        // https://tools.ietf.org/html/rfc6455#section-5.5.1
        if self.shutdown {
            Err(WebSocketError::WebSocketClosedError)
        } else {
            self.send(Frame::Close { payload: None }).await?;
            self.shutdown = true;
            self.stream
                .shutdown()
                .await
                .map_err(|e| WebSocketError::ShutdownError(e))?;
            Ok(())
        }
    }
}
