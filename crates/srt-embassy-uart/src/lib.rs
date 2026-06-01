#![no_std]
#![doc = "Embassy-friendly UART adapter boundary for Serial Realtime Transport."]

mod adapter;
mod link;

pub use adapter::{UartAdapter, UartEventHandler, UartTaskError};
pub use srt_adapter_core::{
    Error, ErrorKind, IoErrorKind, ReceivedMessage, Result, SendFailedEvent, SendFailedReason,
};
