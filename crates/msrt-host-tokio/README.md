# msrt-host-tokio

Tokio host adapter for MSRT.

This crate is the OS-side peer for `msrt-embassy-uart`. It uses
`tokio::io::AsyncRead + AsyncWrite` and exposes the same handler-driven adapter
model:

- `send_message(message)`
- `debug(message)`
- `poll_once_dispatch(now_ms, rx_buf, handler).await`
