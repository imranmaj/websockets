# WebSockets

---

[<img alt="github" src="https://img.shields.io/badge/github-imranmaj/websockets-6bb858?style=for-the-badge&logo=github">](https://github.com/imranmaj/websockets) [<img alt="crates.io" src="https://img.shields.io/crates/v/websockets.svg?style=for-the-badge&color=e38e17&logo=rust">](https://crates.io/crates/websockets) [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-websockets-6f83f2?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K">](https://docs.rs/websockets)

A WebSocket client implementation.

## The `WebSocket` type

The `WebSocket` type manages the WebSocket connection. Use it to connect, send, and receive data. To customize the WebSocket handshake, use a `WebSocketBuilder` (obtained from the `WebSocket::builder()` method).

See the `WebSocket` type for examples on how to use it.

## Frames

Data is sent and received through `Frame`s. If you have a constructed frame you would like to send, you can use the `WebSocket::send()` method; however, there are also convenience methods for each frame type (`send_text()`, `send_binary()`, `close()`, `send_ping()`, and `send_pong()`).

If you have received a `Frame` from which you would like to extract the data, you can use the convenience methods `as_text()`, `as_binary()`, `as_close()`, `as_ping()`, and `as_pong()`. (and their `mut` counterparts), or simply `match`.