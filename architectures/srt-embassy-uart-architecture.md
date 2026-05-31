# srt-embassy-uart Architecture (Phase 1)

## Goal

Provide an Embassy-friendly UART adapter for `srt::Engine` with a simple async API:

- `send(message).await`
- `receive(rx_buf).await`

The adapter itself is protocol I/O glue and does not duplicate SRT core logic.

## Scope (Current)

- one crate: `crates/srt-embassy-uart`
- no board-specific HAL bindings
- no task spawning strategy built-in
- no buffering allocator requirements
- boundary based on `embedded-io-async::{Read, Write}`

## Non-goals (Current)

- no DMA policy abstraction
- no UART interrupt ownership model
- no zero-copy ring protocol yet
- no end-to-end hardware timing benchmark in this phase

## Core Design

`UartDriver<Uart>` owns:

- `uart: Uart`
- `engine: srt::Engine`
- `pending_message: Option<srt::Message>`

Design principle:

1. Application submits outgoing message once via `send`.
2. Adapter queues into `engine.send`.
3. Adapter drains engine events and writes all `Event::Write` bytes to UART.
4. Adapter stores `Event::Message` as pending and returns it from `receive`.
5. Adapter maps protocol and UART failures into one adapter error type.

## Public API

- `UartDriver::new(uart, engine) -> Self`
- `UartDriver::send(&mut self, message: &[u8]) -> async Result<MessageId, UartDriverError<_>>`
- `UartDriver::receive(&mut self, rx_buf: &mut [u8]) -> async Result<Message, UartDriverError<_>>`
- `UartDriver::tick(&mut self, now_ms: u64) -> async Result<(), UartDriverError<_>>`

## Error Model

`UartDriverError<UartError>`:

- `Uart(UartError)` for lower-link I/O failures
- `Protocol(srt::core::Error)` for engine failures (send/receive invariants)
- `SendFailed(srt::SendFailed)` for reliable delivery retry-limit failures

## Runtime Behavior

### send

- non-blocking protocol path: `engine.send(message)` returns immediately
- async I/O path: function awaits while writing generated wire packets to UART

### receive

- loops on UART `read` chunks
- feeds bytes to `engine.receive`
- drains generated writes (ACK/retransmit responses)
- returns exactly one complete SRT `Message` when available

### tick

- bridges external timer into `engine.tick(now_ms)`
- flushes retransmit writes emitted by engine

## Integration Pattern (Embassy)

Typical app model:

- one async task drives `receive`
- app sends with `send` from same ownership context (or guarded by a mutex)
- periodic timer calls `tick` to drive retransmit timeout

Phase 1 keeps ownership single-threaded by design and avoids internal locking.

## Next Steps

- add optional split API for rx/tx task separation
- add board example (e.g., stm32 or rp) with Embassy UART
- add loopback integration test with mock `embedded-io-async` transport
- define backpressure strategy for high-throughput links
