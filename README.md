# WebSockets

A WebSocket client implementation.

## The `WebSocket` type

The `WebSocket` type manages the WebSocket connection. Use it to connect, send, and receive data. To customize the WebSocket handshake, use a `WebSocketBuilder` (obtained from the `WebSocket::builder()` method).

See the `WebSocket` type for examples on how to use it.

## Frames

Data is sent and received through `Frame`s. If you have a constructed frame you would like to send, you can use the `WebSocket::send()` method; however, there are also convenience methods for each frame type (`send_text()`, `send_binary()`, `close()`, `send_ping()`, and `send_pong()`).

If you have received a `Frame` from which you would like to extract the data, you can use the convenience methods `as_text()`, `as_binary()`, `as_close()`, `as_ping()`, and `as_pong()`. (and their `mut` counterparts), or simply `match`.