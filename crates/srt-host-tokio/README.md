# srt-host-tokio

Tokio host adapter for SRT.

This crate is the OS-side peer for `srt-embassy-uart`. It uses `tokio::io::AsyncRead + AsyncWrite` and keeps the same protocol-driving model:

- `send_message(message)`
- `poll_once(now_ms, rx_buf).await`
- `poll_message()`
- `poll_send_failed()`

## Run Example

```sh
cargo run -p srt-host-tokio --example tokio_loopback
```
