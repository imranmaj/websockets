#![forbid(unsafe_code, missing_debug_implementations)]

mod error;
mod websocket;

pub use error::WebSocketError;
pub use websocket::frame::Frame;
pub use websocket::WebSocket;

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
