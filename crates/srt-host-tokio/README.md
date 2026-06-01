# srt-host-tokio

Tokio host adapter for SRT.

This crate is the OS-side peer for `srt-embassy-uart`. It uses
`tokio::io::AsyncRead + AsyncWrite` and exposes the same handler-driven adapter
model:

- `send_message(message)`
- `debug(message)`
- `poll_once_dispatch(now_ms, rx_buf, handler).await`
