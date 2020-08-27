//! A WebSocket client implementation.
//!
//! # The `WebSocket` type
//!
//! The [`WebSocket`] type manages the WebSocket connection.
//! Use it to connect, send, and receive data.
//! To customize the WebSocket handshake, use a [`WebSocketBuilder`]
//! (obtained from the [`WebSocket::builder()`] method).
//!
//! See the [`WebSocket`] type for examples on how to use it.
//!
//! # Frames
//!
//! Data is sent and received through [`Frame`]s. If you have a constructed frame
//! you would like to send, you can use the [`WebSocket::send()`] method;
//! however, there are also convenience methods for each frame type
//! ([`send_text()`](WebSocket::send_text()), [`send_binary()`](WebSocket::send_binary()),
//! [`close()`](WebSocket::close()), [`send_ping()`](WebSocket::send_ping()),
//! and [`send_pong()`](WebSocket::send_pong())).
//!
//! If you have received a [`Frame`] from which you would like to extract the data,
//! you can use the convenience methods [`as_text()`](Frame::as_text()),
//! [`as_binary()`](Frame::as_binary()), [`as_close()`](Frame::as_close()),
//! [`as_ping()`](Frame::as_ping()), and [`as_pong()`](Frame::as_pong()).
//! (and their `mut` counterparts), or simply `match`.

#![forbid(
    unsafe_code,
    missing_debug_implementations,
    missing_crate_level_docs,
    missing_docs
)]

mod error;
mod websocket;

pub use error::WebSocketError;
pub use websocket::frame::Frame;
pub use websocket::{builder::WebSocketBuilder, WebSocket};

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn echo_length_0_to_125() {
        let mut ws = WebSocket::connect("ws://echo.websocket.org/")
            .await
            .unwrap();
        let message = "a".repeat(3).to_string();
        ws.send_text(message.clone(), false, true).await.unwrap();
        let received_frame = ws.receive().await.unwrap();
        let received_message = received_frame.as_text().unwrap().0.clone();
        assert_eq!(message, received_message);
    }

    #[tokio::test]
    async fn echo_length_126_to_u16_max() {
        let mut ws = WebSocket::connect("ws://echo.websocket.org/")
            .await
            .unwrap();
        let message = "a".repeat(300).to_string();
        ws.send_text(message.clone(), false, true).await.unwrap();
        let received_frame = ws.receive().await.unwrap();
        let received_message = received_frame.as_text().unwrap().0.clone();
        assert_eq!(message, received_message);
    }

    #[tokio::test]
    async fn echo_length_u16_max_to_u64_max() {
        let mut ws = WebSocket::connect("ws://echo.websocket.org/")
            .await
            .unwrap();
        let message = "a".repeat(66000).to_string();
        ws.send_text(message.clone(), false, true).await.unwrap();
        let received_frame = ws.receive().await.unwrap();
        let received_message = received_frame.as_text().unwrap().0.clone();
        assert_eq!(message, received_message);
    }

    #[tokio::test]
    async fn echo_tls() {
        let mut ws = WebSocket::connect("wss://echo.websocket.org/")
            .await
            .unwrap();
        let message = "a".repeat(66000).to_string();
        ws.send_text(message.clone(), false, true).await.unwrap();
        let received_frame = ws.receive().await.unwrap();
        let received_message = received_frame.as_text().unwrap().0.clone();
        assert_eq!(message, received_message);
    }

    #[tokio::test]
    async fn close() {
        let mut ws = WebSocket::connect("wss://echo.websocket.org")
            .await
            .unwrap();
        let status_code = ws
            .close(Some((1000, String::new())))
            .await
            .unwrap()
            .as_close()
            .unwrap()
            .0;
        assert_eq!(status_code, 1000);
    }

    #[tokio::test]
    async fn bad_scheme() {
        let resp = WebSocket::connect("http://echo.websocket.org").await;
        if let Ok(_) = resp {
            panic!("expected to fail with bad scheme");
        }
    }
}
