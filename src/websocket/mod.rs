mod builder;
mod parsed_addr;
mod scheme;
mod stream;
mod websocket;

pub use websocket::WebSocket;

// use std::convert::{TryFrom, TryInto};
// use std::io::Error as IoError;
// use std::net::{SocketAddr, ToSocketAddrs};
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use tokio::io::{AsyncRead, AsyncWrite};
// use tokio::net::TcpStream;
// use url::Url;

// use crate::error::WebSocketError;
// use crate::stream::Stream;
// use crate::tls_config::TlsConfig;
// use crate::handshake_config::HandshakeConfig;

// pub struct WebSocketBuilder {
//     tls_config: Option<TlsConfig>,
//     handshake_config: Option<HandshakeConfig>,
// }

// impl WebSocketBuilder {
//     pub fn tls_config(mut self, tls_config: TlsConfig) -> Self {
//         self.tls_config = Some(tls_config);
//         self
//     }

//     pub fn handshake_config(mut self, handshake_config: HandshakeConfig) -> Self {
//         self.handshake_config = Some(handshake_config);
//         self
//     }

//     pub async fn connect(self, addr: &str) -> Result<WebSocket, WebSocketError> {
//         let tls_config = self.tls_config.unwrap_or(TlsConfig::new());
//         let handshake_config = self.handshake_config.unwrap_or(HandshakeConfig::new());
//         WebSocket::connect(addr, tls_config, handshake_config).await
//     }
// }

// #[derive(Debug)]
// pub struct WebSocket {
//     parsed_addr: ParsedAddr,
//     stream: Stream,
// }

// impl AsyncRead for WebSocket {
//     fn poll_read(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &mut [u8],
//     ) -> Poll<Result<usize, IoError>> {
//         unimplemented!();
//     }
// }

// impl AsyncWrite for WebSocket {
//     fn poll_write(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &[u8],
//     ) -> Poll<Result<usize, IoError>> {
//         unimplemented!()
//     }

//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
//         unimplemented!()
//     }

//     fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
//         unimplemented!()
//     }
// }

// #[derive(Debug)]
// pub enum Scheme {
//     Plain,
//     Secure,
// }

// impl TryFrom<&str> for Scheme {
//     type Error = WebSocketError;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "ws" => Ok(Self::Plain),
//             "wss" => Ok(Self::Secure),
//             _ => Err(WebSocketError::SchemeError),
//         }
//     }
// }

// #[derive(Debug)]
// struct ParsedAddr {
//     scheme: Scheme,
//     host: String,
//     addr: SocketAddr,
// }

// impl WebSocket {
//     fn parse_addr(addr: &str) -> Result<ParsedAddr, WebSocketError> {
//         let url = Url::parse(addr).map_err(|e| WebSocketError::ParseError(e))?;
//         let scheme = url.scheme().try_into()?;
//         let host = url.host_str().ok_or(WebSocketError::HostError)?;
//         let port = url
//             .port_or_known_default()
//             .ok_or(WebSocketError::PortError)?;
//         let addr = (host, port)
//             .to_socket_addrs()
//             .map_err(|e| WebSocketError::SocketAddrError(e))?
//             .next()
//             .ok_or(WebSocketError::ResolutionError)?;
//         Ok(ParsedAddr {
//             scheme,
//             host: host.to_owned(),
//             addr,
//         })
//     }

//     pub(crate) async fn connect(addr: &str, tls_config: TlsConfig, handshake_config: HandshakeConfig) -> Result<Self, WebSocketError> {
//         let parsed_addr = Self::parse_addr(addr)?;
//         let stream = Stream::Plain(
//             TcpStream::connect(parsed_addr.addr)
//                 .await
//                 .map_err(|e| WebSocketError::TcpConnectionError(e))?,
//         );
//         let stream = match parsed_addr.scheme {
//             Scheme::Plain => stream,
//             Scheme::Secure => stream.into_tls(&parsed_addr.host, tls_config).await?,
//         };
//         Self { parsed_addr, stream }.handshake(handshake_config).await
//     }

//     async fn handshake(self, handshake_config: HandshakeConfig) -> Result<Self, WebSocketError> {
//         unimplemented!();
//         Ok(self)
//     }

//     fn close() {
//         unimplemented!()
//     }
// }
