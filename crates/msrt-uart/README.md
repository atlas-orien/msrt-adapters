# msrt-uart

UART adapters for MSRT.

This crate has two sides:

- `TokioUartFrontend<T>` for the host/frontend side
- `UartBackend<T>` for MCU/no-std passive backend side

The frontend requires Tokio and is enabled by default. The backend can be built
without default features for MCU firmware.

## Host Frontend

```rust
use msrt_uart::{TokioUartFrontend, UartFrontendEvent};

# async fn run<T>(io: T) -> msrt_uart::TokioResult<()>
# where
#     T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
# {
let mut frontend = TokioUartFrontend::new(io);
frontend.connect()?;
frontend.send(b"hello")?;

loop {
    match frontend.tick().await? {
        UartFrontendEvent::Message(message) => {
            println!("{:?}", message.as_bytes());
        }
        UartFrontendEvent::SendFailed(_) | UartFrontendEvent::TransportUnavailable => {
            frontend.disconnect();
            frontend.connect()?;
        }
        UartFrontendEvent::Idle => {}
    }
}
# }
```

## MCU Backend

Build without Tokio/std:

```sh
cargo build -p msrt-uart --no-default-features --target thumbv7em-none-eabihf
```

Implement `UartIo` for your UART driver:

```rust
use msrt_uart::{UartBackend, UartIo, UartIoError, UartIoResult};

struct BoardUart;

impl UartIo for BoardUart {
    type Error = ();

    fn read(&mut self, buf: &mut [u8]) -> UartIoResult<usize, Self::Error> {
        let _ = buf;
        Err(UartIoError::WouldBlock)
    }

    fn write_all(&mut self, bytes: &[u8]) -> UartIoResult<(), Self::Error> {
        let _ = bytes;
        Ok(())
    }
}

let mut backend = UartBackend::new(BoardUart);
let now_ms = 0;
let _event = backend.tick(now_ms);
```
