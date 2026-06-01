# srt-embassy-uart

Embassy-friendly UART adapter for SRT.

This crate bridges SRT and `embedded-io-async` UART-like drivers.

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

The generated `run(now, rx_buf).await` task owns the protocol loop and should be
started by your runtime during boot.

## Advanced Mode

Use `UartDriver` directly when you need custom scheduling, custom buffering,
tests, or more than one UART link.

- `send_message(message) -> Result<()>`
- `debug(message) -> Result<()>`
- `poll_once(now_ms, rx_buf).await -> Result<()>`
- `poll_message() -> Option<ReceivedMessage>`
- `poll_send_failed() -> Option<SendFailedEvent>`

`send_message` only submits application data to the protocol engine.
`debug` submits data to the SRT log channel.
Reliable delivery is progressed by the background `run` task in Simple Mode, or
by repeated `poll_once` calls in Advanced Mode.

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
use srt_embassy_uart::{Result, UartDriver};

async fn run<U: embedded_io_async::Read + embedded_io_async::Write>(uart: U) -> Result<()> {
    let mut driver = UartDriver::new(uart);
    let mut rx_buf = [0u8; 128];

    driver.send_message(b"hello")?;
    driver.debug(b"boot ok")?;

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

## Run Example

```sh
cargo run -p srt-embassy-uart --example poll_once_loopback
```
