use std::sync::mpsc::{Receiver, Sender};

use rand_chacha::ChaCha20Rng;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};

use super::frame::Frame;
use super::stream::Stream;
use super::FrameType;
#[allow(unused_imports)] // for intra doc links
use super::WebSocket;
use crate::error::WebSocketError;

/// Events sent from the read half to the write half
#[derive(Debug)]
pub(super) enum Event {
    SendPongFrame(Frame),
    SendCloseFrameAndShutdown(Frame),
}

/// The read half of a WebSocket connection, generated from [`WebSocket::split()`].
/// This half can only receive frames.
#[derive(Debug)]
pub struct WebSocketReadHalf {
    pub(super) stream: BufReader<ReadHalf<Stream>>,
    pub(super) last_frame_type: FrameType,
    pub(super) sender: Sender<Event>,
}

impl WebSocketReadHalf {
    /// Receives a [`Frame`] over the WebSocket connection.
    ///
    /// If the received frame is a Ping frame, an event to send a Pong frame will be queued.
    /// If the received frame is a Close frame, an event to send a Close frame
    /// will be queued and the WebSocket will close. However, events are not
    /// acted upon unless flushed (see the documentation on the [`WebSocket`](WebSocket#splitting)
    /// type for more details).
    pub async fn receive(&mut self) -> Result<Frame, WebSocketError> {
        let frame = self.receive_without_handling().await?;
        // handle incoming frames
        match &frame {
            // echo ping frame (https://tools.ietf.org/html/rfc6455#section-5.5.2)
            Frame::Ping { payload } => {
                let pong = Frame::Pong {
                    payload: payload.clone(),
                };
                self.sender
                    .send(Event::SendPongFrame(pong))
                    .map_err(|_e| WebSocketError::ChannelError)?;
            }
            // echo close frame and shutdown (https://tools.ietf.org/html/rfc6455#section-1.4)
            Frame::Close { payload } => {
                let close = Frame::Close {
                    payload: payload
                        .as_ref()
                        .map(|(status_code, _reason)| (status_code.clone(), String::new())),
                };
                self.sender
                    .send(Event::SendCloseFrameAndShutdown(close))
                    .map_err(|_e| WebSocketError::ChannelError)?;
            }
            _ => (),
        }
        Ok(frame)
    }

    /// Receives a [`Frame`] over the WebSocket connection **without handling incoming frames.**
    /// For example, receiving a Ping frame will not queue a Pong frame to be sent,
    /// and receiving a Close frame will not queue a Close frame to be sent nor close
    /// the connection.
    ///
    /// To automatically handle incoming frames, use the [`receive()`](WebSocketReadHalf::receive())
    /// method instead.
    pub async fn receive_without_handling(&mut self) -> Result<Frame, WebSocketError> {
        let frame = Frame::read_from_websocket(self).await?;
        // remember last data frame type in case we get continuation frames (https://tools.ietf.org/html/rfc6455#section-5.2)
        match frame {
            Frame::Text { .. } => self.last_frame_type = FrameType::Text,
            Frame::Binary { .. } => self.last_frame_type = FrameType::Binary,
            _ => (),
        };
        Ok(frame)
    }
}

/// The write half of a WebSocket connection, generated from [`WebSocket::split()`].
/// This half can only send frames.
#[derive(Debug)]
pub struct WebSocketWriteHalf {
    pub(super) shutdown: bool,
    pub(super) sent_closed: bool,
    pub(super) stream: BufWriter<WriteHalf<Stream>>,
    pub(super) rng: ChaCha20Rng,
    pub(super) receiver: Receiver<Event>,
}

impl WebSocketWriteHalf {
    /// Flushes incoming events from the read half. If the read half received a Ping frame,
    /// a Pong frame will be sent. If the read half received a Close frame,
    /// an echoed Close frame will be sent and the WebSocket will close.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn flush(&mut self) -> Result<(), WebSocketError> {
        while let Ok(event) = self.receiver.try_recv() {
            if self.shutdown {
                break;
            }
            match event {
                Event::SendPongFrame(frame) => self.send_without_events_check(frame).await?,
                Event::SendCloseFrameAndShutdown(frame) => {
                    // read half will always send this event if it has received a close frame,
                    // but if we have sent one already, then we have sent and received a close
                    // frame, so we will shutdown
                    if self.sent_closed {
                        self.send_without_events_check(frame).await?;
                        self.shutdown().await?;
                    }
                }
            };
        }
        Ok(())
    }

    /// Sends an already constructed [`Frame`] over the WebSocket connection.
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn send(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        self.flush().await?;
        if self.shutdown || self.sent_closed {
            return Err(WebSocketError::WebSocketClosedError);
        }
        self.send_without_events_check(frame).await
    }

    /// Sends an already constructed [`Frame`] over the WebSocket connection
    /// without flushing incoming events from the read half.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    async fn send_without_events_check(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        frame.send(self).await?;
        Ok(())
    }

    /// Sends a Text frame over the WebSocket connection, constructed
    /// from passed arguments. `continuation` will be `false` and `fin` will be `true`.
    /// To use a custom `continuation` or `fin`, construct a [`Frame`] and use
    /// [`WebSocketWriteHalf::send()`].
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn send_text(&mut self, payload: String) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.6
        self.send(Frame::text(payload)).await
    }

    /// Sends a Binary frame over the WebSocket connection, constructed
    /// from passed arguments. `continuation` will be `false` and `fin` will be `true`.
    /// To use a custom `continuation` or `fin`, construct a [`Frame`] and use
    /// [`WebSocketWriteHalf::send()`].
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn send_binary(&mut self, payload: Vec<u8>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.6
        self.send(Frame::binary(payload)).await
    }

    /// Shuts down the WebSocket connection **without sending a Close frame**.
    /// It is recommended to use the [`close()`](WebSocketWriteHalf::close()) method instead.
    pub async fn shutdown(&mut self) -> Result<(), WebSocketError> {
        self.stream
            .shutdown()
            .await
            .map_err(|e| WebSocketError::ShutdownError(e))?;
        // indicates that a closed frame has been sent, so no more frames should be sent,
        // but the underlying stream is not technically closed (closing the stream
        // would prevent a Close frame from being received by the read half)
        self.sent_closed = true;
        Ok(())
    }

    /// Sends a Close frame over the WebSocket connection, constructed
    /// from passed arguments, and closes the WebSocket connection.
    ///
    /// As per the WebSocket protocol, the server should send a Close frame in response
    /// upon receiving a Close frame. Although the write half will be closed,
    /// the server's echoed Close frame can be read from the still open read half.
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn close(&mut self, payload: Option<(u16, String)>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.1
        self.send(Frame::Close { payload }).await?;
        // self.shutdown().await?;
        Ok(())
    }

    /// Sends a Ping frame over the WebSocket connection, constructed
    /// from passed arguments.
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn send_ping(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.2
        self.send(Frame::Ping { payload }).await
    }

    /// Sends a Pong frame over the WebSocket connection, constructed
    /// from passed arguments.
    ///
    /// This method will flush incoming events.
    /// See the documentation on the [`WebSocket`](WebSocket#splitting) type for more details
    /// about events.
    pub async fn send_pong(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.3
        self.send(Frame::Pong { payload }).await
    }
}
