//! A WebSocket client implementation.
//!
//! [<img alt="github" src="https://img.shields.io/badge/github-imranmaj/websockets-6bb858?style=for-the-badge&logo=github">](https://github.com/imranmaj/websockets) [<img alt="crates.io" src="https://img.shields.io/crates/v/websockets.svg?style=for-the-badge&color=e38e17&logo=rust">](https://crates.io/crates/websockets) [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-websockets-6f83f2?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K">](https://docs.rs/websockets)
//!
//! ```rust
//! # use websockets::WebSocketError;
//! use websockets::WebSocket;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), WebSocketError> {
//! let mut ws = WebSocket::connect("wss://echo.websocket.org/").await?;
//! ws.send_text("foo".to_string()).await?;
//! ws.receive().await?;
//! ws.close(None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! * Simple API
//! * Async/await (tokio runtime)
//! * TLS support (automatically detected)
//!
//! ## Usage
//!
//! The [`WebSocket`] type manages the WebSocket connection.
//! Use it to connect, send, and receive data.
//! Data is sent and received through [`Frame`]s.
//!
//! ## License
//!
//! This project is licensed under the MIT license.

#![forbid(unsafe_code, missing_debug_implementations, missing_docs)]
#![deny(missing_debug_implementations)]

mod error;
mod secure;
mod websocket;

pub use error::WebSocketError;
pub use websocket::frame::Frame;
pub use websocket::split::{WebSocketReadHalf, WebSocketWriteHalf};
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
        ws.send_text(message.clone()).await.unwrap();
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
        ws.send_text(message.clone()).await.unwrap();
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
        ws.send_text(message.clone()).await.unwrap();
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
        ws.send_text(message.clone()).await.unwrap();
        let received_frame = ws.receive().await.unwrap();
        let received_message = received_frame.as_text().unwrap().0.clone();
        assert_eq!(message, received_message);
    }

    #[tokio::test]
    async fn close() {
        let mut ws = WebSocket::connect("wss://echo.websocket.org")
            .await
            .unwrap();
        ws.close(Some((1000, String::new()))).await.unwrap();
        let status_code = ws.receive().await.unwrap().as_close().unwrap().0;
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
