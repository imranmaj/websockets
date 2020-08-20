use native_tls::Error as NativeTlsError;
use rand::Error as RandError;
use std::io::Error as IoError;
use thiserror::Error;
use url::ParseError;
// use crate::websocket::handshake::Handshake;

#[derive(Error, Debug)]
pub enum WebSocketError {
    // TLS configuration errors
    #[error("could not use given TLS configuration")]
    TlsConfigError(NativeTlsError),
    #[error("could not use certificate")]
    CertificateError(NativeTlsError),
    #[error("could not convert certficate to DER")]
    DerConversionError(NativeTlsError),
    #[error("could not parse PKCS #12 archive")]
    IdentityParseError(NativeTlsError),

    // connection errors
    #[error("could not connect using TCP")]
    TcpConnectionError(IoError),
    #[error("could not connect using TLS")]
    TlsConnectionError(NativeTlsError),
    #[error("websocket is already closed")]
    WebSocketClosedError,
    #[error("error shutting down stream")]
    ShutdownError(IoError),

    // handshake errors
    #[error("error invalid handshake accept key from server")]
    KeyError,

    // frame errors
    #[error("control frame has payload larger than 125 bytes")]
    ControlFrameTooLargeError,
    #[error("payload is too large")]
    PayloadTooLargeError,

    // url errors
    #[error("url could not be parsed")]
    ParseError(ParseError),
    #[error(r#"invalid websocket scheme (use "ws" or "wss")"#)]
    SchemeError,
    #[error("invalid or missing host")]
    HostError,
    #[error("invalid or unknown port")]
    PortError,
    #[error("could not parse into SocketAddrs")]
    SocketAddrError(IoError),
    #[error("could not resolve domain")]
    ResolutionError,

    // reading and writing
    #[error("could not read from WebSocket")]
    ReadError(IoError),
    #[error("could not write to WebSocket")]
    WriteError(IoError),

    // other
    // #[error("error using rng")]
    // RngError(RandError),
}
