#![doc = "Tokio host adapter boundary for Serial Realtime Transport."]

mod adapter;

pub use adapter::{HostAdapter, HostTaskError};
pub use srt_adapter_core::{Error, ErrorKind, ReceivedMessage, Result, SendFailedEvent};
