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
        // remember last data frame type in case we get continuation frames (https://tools.ietf.org/html/rfc6455#section-5.2)
        match frame {
            Frame::Text { .. } => self.last_frame_type = FrameType::Text,
            Frame::Binary { .. } => self.last_frame_type = FrameType::Binary,
            _ => (),
        };
        // handle incoming frames
        match &frame {
            // echo ping frame (https://tools.ietf.org/html/rfc6455#section-5.5.2)
            Frame::Ping { payload } => {
                let pong = Frame::Pong {
                    payload: payload.clone(),
                };
                self.send(pong).await?;
            }
            // echo close frame and shutdown (https://tools.ietf.org/html/rfc6455#section-1.4)
            Frame::Close { payload } => {
                let close = Frame::Close {
                    payload: payload
                        .as_ref()
                        .map(|(status_code, _reason)| (status_code.clone(), String::new())),
                };
                self.send(close).await?;
                self.shutdown().await?;
            }
            _ => (),
        }
        Ok(frame)
    }

    pub async fn send_text(
        &mut self,
        payload: String,
        continuation: bool,
        fin: bool,
    ) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.6
        self.send(Frame::Text {
            payload,
            continuation,
            fin,
        })
        .await
    }

    pub async fn send_binary(
        &mut self,
        payload: Vec<u8>,
        continuation: bool,
        fin: bool,
    ) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.6
        self.send(Frame::Binary {
            payload,
            continuation,
            fin,
        })
        .await
    }

    pub async fn close(&mut self, payload: Option<(u16, String)>) -> Result<Frame, WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.1
        if self.shutdown {
            Err(WebSocketError::WebSocketClosedError)
        } else {
            self.send(Frame::Close { payload }).await?;
            let resp = self.receive().await?;
            self.shutdown().await?;
            Ok(resp)
        }
    }

    pub async fn send_ping(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.2
        self.send(Frame::Ping { payload }).await
    }

    pub async fn send_pong(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.3
        self.send(Frame::Pong { payload }).await
    }

    pub async fn shutdown(&mut self) -> Result<(), WebSocketError> {
        self.shutdown = true;
        self.stream
            .shutdown()
            .await
            .map_err(|e| WebSocketError::ShutdownError(e))
    }

    pub fn accepted_subprotocol(&self) -> &Option<String> {
        // https://tools.ietf.org/html/rfc6455#section-1.9
        &self.accepted_subprotocol
    }

    pub fn handshake_response_headers(&self) -> &Option<Vec<(String, String)>> {
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        &self.handshake_response_headers
    }
}
