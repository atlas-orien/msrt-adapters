#![no_std]
#![doc = "Shared adapter primitives for SRT platform integrations."]

mod adapter;
mod error;
mod event;
mod pending_events;

pub use adapter::AdapterCore;
pub use error::{Error, ErrorKind, IoErrorKind, Result};
pub use event::{ReceivedMessage, SendFailedEvent, SendFailedReason};
pub use pending_events::PendingEventHandler;
