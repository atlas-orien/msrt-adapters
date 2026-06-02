#![doc = "Tokio host adapter boundary for Serial Realtime Transport."]

mod adapter;

pub use adapter::{HostAdapter, HostEventHandler, HostTaskError};
pub use msrt_adapter_core::{Error, ErrorKind, ReceivedMessage, Result, SendFailedEvent};
