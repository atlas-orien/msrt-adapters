#![no_std]
#![doc = "Shared adapter primitives for SRT platform integrations."]

mod adapter;
mod driver;
mod error;
mod event;
mod queue;

pub use adapter::AdapterCore;
pub use driver::{AdapterDriver, DriverWrapper};
pub use error::{Error, ErrorKind, IoErrorKind, Result};
pub use event::{ReceivedMessage, SendFailedEvent, SendFailedReason};
pub use queue::{EventQueues, MESSAGE_QUEUE_CAPACITY, SEND_FAILED_QUEUE_CAPACITY};
