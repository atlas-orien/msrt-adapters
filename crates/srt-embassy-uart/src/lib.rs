#![no_std]
#![doc = "Embassy-friendly UART adapter boundary for Serial Realtime Transport."]

mod adapter;
mod singleton;

pub use adapter::{UartAdapter, UartTaskError};
pub use srt_adapter_core::{
    Error, ErrorKind, IoErrorKind, ReceivedMessage, Result, SendFailedEvent, SendFailedReason,
};
