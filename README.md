# msrt-adapters

Platform and runtime adapters for [`msrt`](https://crates.io/crates/msrt).

`msrt` owns the portable no-std protocol engine. This workspace provides small
integration crates for C, std byte streams, UDP, and UART-style links.

## Crates

| Crate | Purpose |
| --- | --- |
| `msrt-ffi` | C ABI bindings and a portable header for host or MCU builds. |
| `msrt-std` | Blocking `std::io::Read + Write` frontend/backend adapters. |
| `msrt-udp` | Tokio UDP client/server adapters with reconnect-friendly events. |
| `msrt-uart` | Tokio host UART frontend plus no-std MCU backend traits. |

## Build

```sh
cargo test --workspace
```

Build no-std UART backend code:

```sh
cargo build -p msrt-uart --no-default-features
```

Build the C ABI static library for a specific target:

```sh
cargo build -p msrt-ffi --release --target thumbv7em-none-eabihf --no-default-features
```

Each target needs its own build artifact. There is no universal static or
dynamic library that works across desktop OS targets and MCU targets.
