# srt-embassy-uart

Embassy-friendly UART adapter for SRT.

This crate bridges `srt::Engine` and `embedded-io-async` UART-like drivers with a `poll_once` model.

## API

- `send_message(message) -> Result<MessageId>`
- `poll_once(now_ms, rx_buf).await -> Result<()>`
- `poll_message() -> Option<Message>`
- `poll_send_failed() -> Option<SendFailed>`

`send_message` only submits data to the protocol engine. Reliable delivery is progressed by repeated `poll_once` calls.

## Error Model

This crate defines a unified boundary:

- `ErrorKind`
- `Error`
- `Result<T>`

Queue overflow is explicitly reported via:

- `ErrorKind::MessageQueueFull`
- `ErrorKind::SendFailedQueueFull`

## Minimal Usage

```rust
use srt::{Config, Engine};
use srt_embassy_uart::{Result, UartDriver};

async fn run<U: embedded_io_async::Read + embedded_io_async::Write>(uart: U) -> Result<()> {
    let engine = Engine::new(Config::default());
    let mut driver = UartDriver::new(uart, engine);
    let mut rx_buf = [0u8; 128];

    let _message_id = driver.send_message(b"hello")?;

    loop {
        driver.poll_once(1000, &mut rx_buf).await?;

        while let Some(message) = driver.poll_message() {
            let payload = message.as_bytes();
            let _ = payload;
        }

        while let Some(failed) = driver.poll_send_failed() {
            let _ = failed;
        }
    }
}
```
