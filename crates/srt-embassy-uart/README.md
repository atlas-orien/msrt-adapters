# msrt-embassy-uart

Embassy-friendly UART adapter for MSRT.

This crate bridges MSRT and `embedded-io-async` UART-like adapters.

## Simple Mode

Most MCU applications use one UART link to the host. Define a UART link API once:

```rust
mod msrt_uart {
    use crate::MyUart;

    msrt_embassy_uart::define_msrt_uart!(MyUart);
}
```

Application code initializes the UART and sends messages through the generated API:

```rust
msrt_uart::init(uart)?;
msrt_uart::send_message(b"hello")?;
msrt_uart::debug(b"boot ok")?;
```

The generated `run_task` owns the protocol loop. User code only implements a handler:

```rust
use msrt_embassy_uart::{ReceivedMessage, UartEventHandler, UartTaskError};

struct App;

impl UartEventHandler for App {
    fn handle_message(&mut self, message: ReceivedMessage) {
        let payload = message.as_bytes();
        let _ = payload;
    }

    fn handle_error(&mut self, error: UartTaskError) {
        let _ = error;
    }
}

let mut app = App;
msrt_uart::run_task(now, &mut rx_buf, &mut app).await;
```

## Advanced Mode

Use `UartAdapter` directly when you need custom scheduling, custom buffering,
tests, or more than one UART link.

- `send_message(message) -> Result<()>`
- `debug(message) -> Result<()>`
- `poll_once(now_ms, rx_buf).await -> Result<()>`
- `poll_once_dispatch(now_ms, rx_buf, handler).await`

`send_message` only submits application data to the protocol engine.
`debug` submits data to the MSRT log channel.
Reliable delivery is progressed by the background `run_task` in Simple Mode, or
by repeated `poll_once_dispatch` calls in Advanced Mode.

## Error Model

This crate defines a unified boundary:

- `ErrorKind`
- `Error`
- `Result<T>`

Pending event overflow is explicitly reported via:

- `ErrorKind::MessageEventsFull`
- `ErrorKind::SendFailedEventsFull`

## Advanced Usage

```rust
use msrt_embassy_uart::{ReceivedMessage, Result, UartAdapter, UartEventHandler, UartTaskError};

struct App;

impl UartEventHandler for App {
    fn handle_message(&mut self, message: ReceivedMessage) {
        let payload = message.as_bytes();
        let _ = payload;
    }

    fn handle_error(&mut self, error: UartTaskError) {
        let _ = error;
    }
}

async fn run<U: embedded_io_async::Read + embedded_io_async::Write>(uart: U) -> Result<()> {
    let mut adapter = UartAdapter::new(uart);
    let mut app = App;
    let mut rx_buf = [0u8; 128];

    adapter.send_message(b"hello")?;
    adapter.debug(b"boot ok")?;

    loop {
        adapter.poll_once_dispatch(1000, &mut rx_buf, &mut app).await;
    }
}
```

## Run Example

```sh
cargo run -p msrt-embassy-uart --features std --example mcu_uart_link
```
