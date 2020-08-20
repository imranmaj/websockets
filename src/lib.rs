#![forbid(unsafe_code)]

mod error;
// mod tls_config;
// mod handshake_config;
mod websocket;

pub use websocket::WebSocket;
pub use error::WebSocketError;
// pub use tls_config::{Identity, Certificate, Protocol};
pub use websocket::builder::{Identity, Certificate, Protocol};

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
        match resp {
            Ok(_) => panic!("expected to fail with bad scheme"),
            Err(e) => println!("{}", e),
        }
    }
}
