# srt-embassy-uart

Embassy-friendly UART adapter for SRT.

This crate bridges SRT and `embedded-io-async` UART-like adapters.

## Simple Mode

Most MCU applications use one UART link to the host. Define a singleton API once:

```rust
mod srt_uart {
    use crate::MyUart;

    srt_embassy_uart::define_srt_uart!(MyUart);
}
```

Application code only needs the simple functions:

```rust
srt_uart::init(uart)?;
srt_uart::send_message(b"hello")?;
srt_uart::debug(b"boot ok")?;
```

The generated `run_task` owns the protocol loop. User code only provides
message and error handlers:

```rust
srt_uart::run_task(
    now,
    &mut rx_buf,
    |message| {
        let payload = message.as_bytes();
        let _ = payload;
    },
    |error| {
        let _ = error;
    },
)
.await;
```

## Advanced Mode

Use `UartAdapter` directly when you need custom scheduling, custom buffering,
tests, or more than one UART link.

- `send_message(message) -> Result<()>`
- `debug(message) -> Result<()>`
- `poll_once(now_ms, rx_buf).await -> Result<()>`
- `poll_once_dispatch(now_ms, rx_buf, handle_message, handle_error).await`

`send_message` only submits application data to the protocol engine.
`debug` submits data to the SRT log channel.
Reliable delivery is progressed by the background `run_task` in Simple Mode, or
by repeated `poll_once_dispatch` calls in Advanced Mode.

## Error Model

This crate defines a unified boundary:

- `ErrorKind`
- `Error`
- `Result<T>`

Queue overflow is explicitly reported via:

- `ErrorKind::MessageQueueFull`
- `ErrorKind::SendFailedQueueFull`

## Advanced Usage

```rust
use srt_embassy_uart::{Result, UartAdapter};

async fn run<U: embedded_io_async::Read + embedded_io_async::Write>(uart: U) -> Result<()> {
    let mut adapter = UartAdapter::new(uart);
    let mut rx_buf = [0u8; 128];

    adapter.send_message(b"hello")?;
    adapter.debug(b"boot ok")?;

    loop {
        adapter
            .poll_once_dispatch(
                1000,
                &mut rx_buf,
                |message| {
            let payload = message.as_bytes();
            let _ = payload;
                },
                |error| {
                    let _ = error;
                },
            )
            .await;
    }
}
```

## Run Example

```sh
cargo run -p srt-embassy-uart --example poll_once_loopback
```
