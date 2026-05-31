# srt-embassy-uart

Embassy-friendly UART adapter for SRT.

This crate bridges `srt::Engine` and `embedded-io-async` UART-like drivers.

## API

- `send(message).await -> Result<MessageId>`
- `receive(rx_buf).await -> Result<Message>`
- `tick(now_ms).await -> Result<()>`

All public methods use this crate's unified `Result<T>`.

## Error Model

Like `srt-error`, this crate defines its own error boundary:

- `ErrorKind`
- `Error`
- `Result<T>`

`Error` can carry:

- UART failure class (`UartErrorKind`) for read/write/flush
- underlying SRT protocol error (`srt::core::Error`)
- reliable send failure details (`srt::SendFailed`)

## Minimal Usage

```rust
use srt::{Config, Engine};
use srt_embassy_uart::{Result, UartDriver};

async fn run<U: embedded_io_async::Read + embedded_io_async::Write>(uart: U) -> Result<()> {
    let engine = Engine::new(Config::default());
    let mut driver = UartDriver::new(uart, engine);

    let _message_id = driver.send(b"hello").await?;

    let mut rx_buf = [0u8; 128];
    let message = driver.receive(&mut rx_buf).await?;
    let payload = message.as_bytes();

    driver.tick(1000).await?;

    let _ = payload;
    Ok(())
}
```
