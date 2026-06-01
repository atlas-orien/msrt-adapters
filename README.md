# srt-adapters

Platform and runtime adapters for SRT.

This repository is intentionally separate from the core `srt` protocol repository. The `srt` crate defines the no-std message transport protocol. This repository is for platform adapters, runtime adapters, and integration experiments.

Current status: scaffold only. The first adapter crate is `srt-embassy-uart`; it does not target a specific MCU board yet.

## Design

- keep SRT protocol logic in `srt`
- keep platform integration here
- avoid binding to a specific STM32/RP/ESP HAL in the first version
- use `embedded-io-async` traits as the adapter boundary

## Crates

```text
crates/srt-embassy-uart
crates/srt-host-tokio
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
