pub mod builder;
pub mod frame;
mod handshake;
mod parsed_addr;
mod stream;

use futures::FutureExt;
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

/// Manages the WebSocket connection; used to connect, send data, and receive data.
///
/// Connect directly with [`WebSocket::connect()`]...
///
/// ```
/// use websockets::WebSocket;
///
/// #[tokio::main]
/// async fn main() {
///     let mut ws = WebSocket::connect("wss://echo.websocket.org/")
///         .await
///         .unwrap();
/// }
/// ```
///
/// ...or customize the handshake using a [`WebSocketBuilder`] obtained from [`WebSocket::builder()`].
///
/// ```
/// # use websockets::WebSocket;
/// # #[tokio::main]
/// # async fn main() {
/// let mut ws = WebSocket::builder()
///     .add_subprotocol("wamp")
///     .connect("wss://echo.websocket.org")
///     .await
///     .unwrap();
/// # }
/// ```
///
/// Use the `WebSocket::send*` methods to send frames...
///
/// ```
/// # use websockets::WebSocket;
/// # #[tokio::main]
/// # async fn main() {
/// # let mut ws = WebSocket::connect("wss://echo.websocket.org")
/// #     .await
/// #     .unwrap();
/// ws.send_text("foo".to_string(), false, true).await.unwrap();
/// # }
/// ```
///
/// ...and [`WebSocket::receive()`] to receive frames.
///
/// ```
/// # use websockets::WebSocket;
/// # #[tokio::main]
/// # async fn main() {
/// # let mut ws = WebSocket::connect("wss://echo.websocket.org")
/// #     .await
/// #     .unwrap();
/// # ws.send_text("foo".to_string(), false, true).await.unwrap();
/// let received_frame = ws.receive().await.unwrap();
/// let received_msg = received_frame.as_text().unwrap().0.clone();
/// assert_eq!(received_msg, "foo".to_string()); // echo.websocket.org echoes text frames
/// # }
/// ```
///
/// Finally, close the connection with [`WebSocket::close()`].
///
/// ```
/// # use websockets::WebSocket;
/// # #[tokio::main]
/// # async fn main() {
/// #     let mut ws = WebSocket::connect("wss://echo.websocket.org")
/// #         .await
/// #         .unwrap();
/// let status_code = ws.close(Some((1000, String::new())))
///     .await
///     .unwrap()
///     .as_close()
///     .unwrap()
///     .0;
/// assert_eq!(status_code, 1000);
/// # }
/// ```
#[derive(Debug)]
pub struct WebSocket {
    stream: BufStream<Stream>,
    shutdown: bool,
    last_frame_type: FrameType,
    accepted_subprotocol: Option<String>,
    handshake_response_headers: Option<Vec<(String, String)>>,
}

impl WebSocket {
    /// Constructs a [`WebSocketBuilder`], which can be used to customize
    /// the WebSocket handshake.
    pub fn builder() -> WebSocketBuilder {
        WebSocketBuilder::new()
    }

    /// Connects to a URL (and performs the WebSocket handshake).
    pub async fn connect(url: &str) -> Result<Self, WebSocketError> {
        WebSocketBuilder::new().connect(url).await
    }

    /// Sends an already constructed [`Frame`] over the WebSocket connection.
    pub async fn send(&mut self, frame: Frame) -> Result<(), WebSocketError> {
        if self.shutdown {
            return Err(WebSocketError::WebSocketClosedError);
        }
        frame.send(self).await?;
        Ok(())
    }

    /// Checks whether a frame is ready to be received.
    pub fn ready_to_receive(&mut self) -> Result<bool, WebSocketError> {
        let tcp_stream = self.stream.get_mut().get_mut();
        match tcp_stream.peek(&mut vec![0]).now_or_never() {
            Some(Ok(bytes_read)) => Ok(bytes_read > 0),
            Some(Err(e)) => Err(WebSocketError::ReadError(e)),
            None => Ok(false),
        }
    }

    /// Receives a [`Frame`] over the WebSocket connection.
    ///
    /// If the received frame is a Ping frame, a Pong frame will be sent.
    /// If the received frame is a Close frame, an echoed Close frame
    /// will be sent and the WebSocket will close.
    /// 
    /// This method may block until receiving is ready. To check whether
    /// data is available to be received, use [`ready_to_receive()`](WebSocket::ready_to_receive()).
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

    /// Sends a Text frame over the WebSocket connection, constructed
    /// from passed arguments.
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

    /// Sends a Binary frame over the WebSocket connection, constructed
    /// from passed arguments.
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

    /// Sends a Close frame over the WebSocket connection, constructed
    /// from passed arguments, and closes the WebSocket connection.
    /// This method will attempt to wait for an echoed Close frame,
    /// which is returned.
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

    /// Sends a Ping frame over the WebSocket connection, constructed
    /// from passed arguments.
    pub async fn send_ping(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.2
        self.send(Frame::Ping { payload }).await
    }

    /// Sends a Pong frame over the WebSocket connection, constructed
    /// from passed arguments.
    pub async fn send_pong(&mut self, payload: Option<Vec<u8>>) -> Result<(), WebSocketError> {
        // https://tools.ietf.org/html/rfc6455#section-5.5.3
        self.send(Frame::Pong { payload }).await
    }

    /// Shuts down the WebSocket connection **without sending a Close frame**.
    /// It is recommended to use the [`close()`](WebSocket::close()) method instead.
    pub async fn shutdown(&mut self) -> Result<(), WebSocketError> {
        self.shutdown = true;
        self.stream
            .shutdown()
            .await
            .map_err(|e| WebSocketError::ShutdownError(e))
    }

    /// Returns the subprotocol that was accepted by the server during the handshake,
    /// if any.
    pub fn accepted_subprotocol(&self) -> &Option<String> {
        // https://tools.ietf.org/html/rfc6455#section-1.9
        &self.accepted_subprotocol
    }

    /// Returns the headers that were returned by the server during the handshake.
    pub fn handshake_response_headers(&self) -> &Option<Vec<(String, String)>> {
        // https://tools.ietf.org/html/rfc6455#section-4.2.2
        &self.handshake_response_headers
    }
}
