# msrt-ffi

C ABI bindings for MSRT.

This crate is intended to be shipped as source. C users build a target-specific
static library from this crate, then link that library into their C application
or MCU firmware.

```text
msrt_ffi.h is portable.
libmsrt_ffi.a is target-specific.
```

There is no universal `.a`, `.so`, or `.dylib` that works on every host and MCU.

## What The C Side Owns

`msrt-ffi` does not open serial ports, sockets, USB devices, or UART drivers.
The C application owns real IO:

```text
transport read bytes  -> msrt_endpoint_receive()
msrt_endpoint_poll()  -> write MSRT_POLL_TRANSMIT bytes to transport
MSRT_POLL_MESSAGE     -> deliver application payload
```

## Build For Host

Build a host static library:

```sh
cargo build -p msrt-ffi --release
```

Outputs:

```text
target/release/libmsrt_ffi.a
crates/msrt-ffi/include/msrt_ffi.h
```

Example macOS link command:

```sh
cc app.c \
  -I crates/msrt-ffi/include \
  target/release/libmsrt_ffi.a \
  -framework Security -framework CoreFoundation \
  -o app
```

Example Linux link command:

```sh
cc app.c \
  -I crates/msrt-ffi/include \
  target/release/libmsrt_ffi.a \
  -ldl -lpthread -lm \
  -o app
```

Native libraries vary by toolchain and target.

## Build For MCU

Build a no-std static library for a specific MCU target:

```sh
rustup target add thumbv7em-none-eabihf

cargo build -p msrt-ffi \
  --release \
  --target thumbv7em-none-eabihf \
  --no-default-features
```

When compiling C code against a no-heap MCU build, define `MSRT_FFI_NO_HEAP` so
the host-only `msrt_endpoint_new/free` declarations are hidden:

```sh
arm-none-eabi-gcc -DMSRT_FFI_NO_HEAP ...
```

Output:

```text
target/thumbv7em-none-eabihf/release/libmsrt_ffi.a
```

That `.a` is only for that target. A Cortex-M0, Cortex-M4 hard-float, RISC-V MCU,
Linux host, and macOS host each need their own build.

The MCU C firmware link step usually looks like:

```text
C application objects
+ vendor HAL/startup objects
+ target/<mcu-target>/release/libmsrt_ffi.a
+ linker script
-> firmware.elf
-> firmware.bin / firmware.hex
```

Use your MCU toolchain's linker flags for section garbage collection, for
example `-ffunction-sections`, `-fdata-sections`, and `-Wl,--gc-sections`.

## MCU-Friendly Storage API

The primary API does not require heap allocation. C owns endpoint storage:

```c
#include "msrt_ffi.h"

msrt_endpoint_storage_t storage;
msrt_endpoint_t *endpoint = NULL;

int32_t status = msrt_endpoint_init(&storage, NULL, &endpoint);
if (status != MSRT_STATUS_OK) {
    return 1;
}

/* Use endpoint. */

msrt_endpoint_deinit(endpoint);
```

`msrt_endpoint_storage_t` is opaque. Do not read or write its fields directly.

The header defines conservative storage for the endpoint. At runtime or startup,
you may compare:

```c
size_t need = msrt_endpoint_storage_size();
size_t align = msrt_endpoint_storage_align();
```

If the storage compiled into the header is not enough for this Rust build,
`msrt_endpoint_init()` returns `MSRT_STATUS_INVALID_STORAGE`.

## Host Convenience API

When built with default features, host programs may also use heap allocation:

```c
msrt_endpoint_t *endpoint = msrt_endpoint_new(NULL);
msrt_endpoint_free(endpoint);
```

This API is not available in `--no-default-features` MCU builds. Portable C code
should prefer `msrt_endpoint_init()` and `msrt_endpoint_deinit()`.

## Endpoint Roles

Use `MSRT_ROLE_CLIENT` for the active side. A client must call
`msrt_endpoint_connect()` before sending messages.

Use `MSRT_ROLE_PASSIVE` for a single-peer passive side, such as an MCU waiting
for a host. A passive endpoint creates its protocol session when bytes arrive,
so it does not call `msrt_endpoint_connect()`.

## Custom Configuration

```c
msrt_config_t config;
msrt_config_default(&config);

config.role = MSRT_ROLE_PASSIVE;
config.integrity = MSRT_INTEGRITY_CRC32;
config.retransmit_timeout_ms = 100;

msrt_endpoint_storage_t storage;
msrt_endpoint_t *endpoint = NULL;
msrt_endpoint_init(&storage, &config, &endpoint);
```

Both peers must use the same integrity setting.

## Driving The Endpoint

Call `receive` whenever bytes arrive. Input does not need to be packet-aligned.

```c
uint8_t rx[256];
size_t rx_len = transport_read(rx, sizeof(rx));

if (rx_len > 0) {
    msrt_receive_event_t receive_event;
    msrt_endpoint_receive(endpoint, now_ms, rx, rx_len, &receive_event);
}
```

Queue an application message:

```c
const uint8_t payload[] = "hello";
msrt_endpoint_send(endpoint, payload, sizeof(payload) - 1);
```

Poll until idle. MSRT returns one action per call.

```c
msrt_poll_event_t event;

while (msrt_endpoint_poll(endpoint, now_ms, &event) == MSRT_STATUS_OK) {
    if (event.kind == MSRT_POLL_IDLE) {
        break;
    }

    if (event.kind == MSRT_POLL_TRANSMIT) {
        transport_write(event.bytes, event.len);
    } else if (event.kind == MSRT_POLL_MESSAGE) {
        handle_message(event.bytes, event.len);
    } else if (event.kind == MSRT_POLL_SEND_FAILED) {
        msrt_endpoint_disconnect(endpoint);
        break;
    }
}
```

## Main Loop Shape

```c
while (running) {
    uint64_t now_ms = monotonic_milliseconds();

    uint8_t rx[256];
    size_t rx_len = transport_read(rx, sizeof(rx));
    if (rx_len > 0) {
        msrt_receive_event_t receive_event;
        msrt_endpoint_receive(endpoint, now_ms, rx, rx_len, &receive_event);
    }

    msrt_poll_event_t event;
    while (msrt_endpoint_poll(endpoint, now_ms, &event) == MSRT_STATUS_OK) {
        if (event.kind == MSRT_POLL_IDLE) {
            break;
        }
        if (event.kind == MSRT_POLL_TRANSMIT) {
            transport_write(event.bytes, event.len);
        }
        if (event.kind == MSRT_POLL_MESSAGE) {
            handle_message(event.bytes, event.len);
        }
    }
}
```

## Status Codes

```text
MSRT_STATUS_OK              success
MSRT_STATUS_NULL            required pointer was NULL
MSRT_STATUS_INVALID_SLICE   byte pointer was NULL while length was non-zero
MSRT_STATUS_PROTOCOL_ERROR  MSRT rejected or could not process input
MSRT_STATUS_UNSUPPORTED     unsupported role or option
MSRT_STATUS_NO_SESSION      send was requested without an active session
MSRT_STATUS_INVALID_STORAGE storage is too small or incorrectly aligned
```

## Example

`examples/loopback.c` shows a small in-process client/passive exchange. It feeds
`MSRT_POLL_TRANSMIT` bytes from one endpoint into the other endpoint. In a real
program, replace that transfer with UART, USB CDC, socket, or another byte
transport.
