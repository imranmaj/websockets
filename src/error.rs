use native_tls::Error as NativeTlsError;
use std::io::Error as IoError;
use thiserror::Error;
use url::ParseError;

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
    #[error("error performing WebSocket handshake")]
    HandshakeError,

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
}
