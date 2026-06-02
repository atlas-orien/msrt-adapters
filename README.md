# msrt-adapters

Platform and runtime adapters for MSRT.

This repository is intentionally separate from the core `msrt` protocol repository. The `msrt` crate defines the no-std message transport protocol. This repository is for platform adapters, runtime adapters, and integration experiments.

Current status: scaffold only. The first adapter crate is `msrt-embassy-uart`; it does not target a specific MCU board yet.

## Design

- keep MSRT protocol logic in `msrt`
- keep platform integration here
- avoid binding to a specific STM32/RP/ESP HAL in the first version
- use `embedded-io-async` traits as the adapter boundary

## Crates

```text
crates/msrt-embassy-uart
crates/msrt-host-tokio
```

Embassy-friendly UART adapter boundary built on `embedded-io-async`.
Tokio host adapter boundary built on OS async I/O.

## Run

```sh
cargo check --workspace
```

## Future Work

- add a board-specific Embassy example
- define ring-buffer ownership patterns
- test UART read/write chunking
- validate timing on real hardware
