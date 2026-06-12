//! Adapter events.

use msrt::endpoint::{MessageEvent, SendFailedEvent};

/// High-level event produced by a std adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterEvent {
    /// A complete application message was received.
    Message(MessageEvent),
    /// Reliable send failed.
    SendFailed(SendFailedEvent),
    /// No action is currently pending.
    Idle,
}
