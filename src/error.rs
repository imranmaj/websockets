use native_tls::Error as NativeTlsError;
use std::io::Error as IoError;
use thiserror::Error;
use url::ParseError;

/// The possible error types from the WebSocket connection.
#[derive(Error, Debug)]
pub enum WebSocketError {
    // connection errors
    /// Error connecting using TCP
    #[error("could not connect using TCP")]
    TcpConnectionError(IoError),
    /// Error connecting using TLS
    #[error("could not connect using TLS")]
    TlsConnectionError(NativeTlsError),
    /// Attempted to use the WebSocket when it is already closed
    #[error("websocket is already closed")]
    WebSocketClosedError,
    /// Error shutting down the internal stream
    #[error("error shutting down stream")]
    ShutdownError(IoError),

    // handshake errors
    /// Invalid handshake response from the server
    #[error("invalid handshake response")]
    InvalidHandshakeError,
    /// The server rejected the handshake request
    #[error("server rejected handshake")]
    HandshakeFailedError {
        /// Status code from the server's handshake response
        status_code: String,
        /// Headers from the server's handshake response
        headers: Vec<(String, String)>,
        /// Body of the server's handshake response, if any
        body: Option<String>,
    },

    // frame errors
    /// Attempted to use a control frame whose payload is more than 125 bytes
    #[error("control frame has payload larger than 125 bytes")]
    ControlFrameTooLargeError,
    /// Attempted to use a frame whose payload is too large
    #[error("payload is too large")]
    PayloadTooLargeError,
    /// Received an invalid frame
    #[error("received frame is invalid")]
    InvalidFrameError,
    /// Received a masked frame from the server
    #[error("received masked frame")]
    ReceivedMaskedFrameError,

    // url errors
    /// URL could not be parsed
    #[error("url could not be parsed")]
    ParseError(ParseError),
    /// URL has invalid WebSocket scheme (use "ws" or "wss")
    #[error(r#"invalid websocket scheme (use "ws" or "wss")"#)]
    SchemeError,
    /// URL host is invalid or missing
    #[error("invalid or missing host")]
    HostError,
    /// URL port is invalid
    #[error("invalid or unknown port")]
    PortError,
    /// Could not parse URL into SocketAddrs
    #[error("could not parse into SocketAddrs")]
    SocketAddrError(IoError),
    /// Could not resolve the URL's domain
    #[error("could not resolve domain")]
    ResolutionError,

    // reading and writing
    /// Error reading from WebSocket
    #[error("could not read from WebSocket")]
    ReadError(IoError),
    /// Error writing to WebSocket
    #[error("could not write to WebSocket")]
    WriteError(IoError),
}
