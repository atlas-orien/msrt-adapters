# msrt-std

Blocking std byte-stream adapters for MSRT.

This crate is for host-side Rust programs that already own a byte stream:

- serial ports
- TCP streams
- pipes
- test doubles implementing `Read + Write`

The first version provides:

- `StdFrontend<T>`: active client/frontend endpoint
- `StdBackend<T>`: passive single-peer backend endpoint

`T` only needs to implement `std::io::Read + std::io::Write`.

## Basic Frontend

```rust
use msrt_std::{AdapterEvent, StdFrontend};

# fn run<T: std::io::Read + std::io::Write>(io: T) -> msrt_std::Result<()> {
let mut frontend = StdFrontend::new(io);
frontend.connect()?;
frontend.send(b"hello")?;

loop {
    match frontend.tick()? {
        AdapterEvent::Message(message) => {
            println!("{:?}", message.as_bytes());
        }
        AdapterEvent::SendFailed(_) => {
            frontend.disconnect();
        }
        AdapterEvent::Idle => {}
    }
}
# }
```

Use nonblocking IO when calling `receive_available` or `tick`; `WouldBlock` is
treated as "no input right now".
