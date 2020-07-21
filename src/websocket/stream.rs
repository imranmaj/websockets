use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector as TokioTlsConnector, TlsStream};

use crate::error::WebSocketError;
use crate::tls_config::TlsConfig;

#[derive(Debug)]
pub enum Stream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Stream {
    pub async fn into_tls(self, host: &str, tls_config: TlsConfig) -> Result<Self, WebSocketError> {
        match self {
            Self::Plain(tcp_stream) => {
                let connector: TokioTlsConnector = tls_config.build()?.into();
                let tls_stream = connector.connect(host, tcp_stream).await.map_err(|e| WebSocketError::TlsConnectionError(e))?;
                Ok(Stream::Tls(tls_stream))
            }
            Self::Tls(_) => Ok(self),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, IoError>> {
        match self.get_mut() {
            Self::Plain(tcp_stream) => Pin::new(tcp_stream).poll_read(cx, buf),
            Self::Tls(tls_stream) => Pin::new(tls_stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        match self.get_mut() {
            Self::Plain(tcp_stream) => Pin::new(tcp_stream).poll_write(cx, buf),
            Self::Tls(tls_stream) => Pin::new(tls_stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        match self.get_mut() {
            Self::Plain(tcp_stream) => Pin::new(tcp_stream).poll_flush(cx),
            Self::Tls(tls_stream) => Pin::new(tls_stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), IoError>> {
        match self.get_mut() {
            Self::Plain(tcp_stream) => Pin::new(tcp_stream).poll_shutdown(cx),
            Self::Tls(tls_stream) => Pin::new(tls_stream).poll_shutdown(cx),
        }
    }
}
