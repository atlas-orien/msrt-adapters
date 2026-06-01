#![no_std]
#![doc = "Embassy-friendly UART adapter boundary for Serial Realtime Transport."]

mod driver;
mod io_pump;
mod singleton;

pub use driver::UartDriver;
pub use srt_adapter_core::{
    Error, ErrorKind, IoErrorKind, ReceivedMessage, Result, SendFailedEvent, SendFailedReason,
};
