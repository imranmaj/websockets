use native_tls::TlsConnector as NativeTlsTlsConnector;
use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector as TokioTlsConnector, TlsStream};

use crate::error::WebSocketError;

#[derive(Debug)]
pub(super) enum Stream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Stream {
    pub(super) async fn into_tls(
        self,
        host: &str,
        tls_connector: NativeTlsTlsConnector,
    ) -> Result<Self, WebSocketError> {
        match self {
            Self::Plain(tcp_stream) => {
                let connector: TokioTlsConnector = tls_connector.into();
                let tls_stream = connector
                    .connect(host, tcp_stream)
                    .await
                    .map_err(|e| WebSocketError::TlsConnectionError(e))?;
                Ok(Stream::Tls(tls_stream))
            }
            Self::Tls(_) => Ok(self),
        }
    }

    // pub(super) fn get_ref(&self) -> &TcpStream {
    //     match self {
    //         Self::Plain(tcp_stream) => tcp_stream,
    //         Self::Tls(tls_stream) => tls_stream.get_ref().get_ref().get_ref(),
    //     }
    // }

    // pub(super) fn get_mut(&mut self) -> &mut TcpStream {
    //     match self {
    //         Self::Plain(tcp_stream) => tcp_stream,
    //         Self::Tls(tls_stream) => tls_stream.get_mut().get_mut().get_mut(),
    //     }
    // }
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
