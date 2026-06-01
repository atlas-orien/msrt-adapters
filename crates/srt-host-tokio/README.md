# srt-host-tokio

Tokio host adapter for SRT.

This crate is the OS-side peer for `srt-embassy-uart`. It uses `tokio::io::AsyncRead + AsyncWrite` and keeps the same protocol-driving model:

- `send_message(message)`
- `debug(message)`
- `poll_once_dispatch(now_ms, rx_buf, handle_message, handle_error).await`

## Run Example

```sh
cargo run -p srt-host-tokio --example tokio_loopback
```

QEMU TCP serial client:

```sh
cargo run -p srt-host-tokio --example qemu_tcp_client
```
