#![forbid(unsafe_code)]

mod error;
mod websocket;

pub use error::WebSocketError;
pub use websocket::frame::Frame;
pub use websocket::WebSocket;
// pub use websocket::builder::{Certificate, Identity, Protocol};

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn echo() {
        use crate::WebSocket;

        WebSocket::connect("ws://echo.websocket.org/")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn echo_tls() {
        use crate::WebSocket;

        WebSocket::connect("wss://echo.websocket.org/")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn bad_scheme() {
        use crate::WebSocket;

        let resp = WebSocket::connect("http://echo.websocket.org").await;
        if let Ok(_) = resp {
            panic!("expected to fail with bad scheme");
        }
    }
}
