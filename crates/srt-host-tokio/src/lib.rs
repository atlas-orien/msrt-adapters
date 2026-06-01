#![doc = "Tokio host adapter boundary for Serial Realtime Transport."]

mod driver;
mod io_pump;
mod send;

pub use driver::HostDriver;
pub use srt_adapter_core::{Error, ErrorKind, ReceivedMessage, Result, SendFailedEvent};
