//! UART adapter events.

use msrt::endpoint::{MessageEvent, SendFailedEvent};

/// High-level event produced by the Tokio host UART frontend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartFrontendEvent {
    /// A complete application message was received.
    Message(MessageEvent),
    /// Reliable send failed.
    SendFailed(SendFailedEvent),
    /// Transport is currently unavailable.
    TransportUnavailable,
    /// No action is currently pending.
    Idle,
}

/// High-level event produced by the MCU/passive UART backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartBackendEvent {
    /// A complete application message was received.
    Message(MessageEvent),
    /// Reliable send failed.
    SendFailed(SendFailedEvent),
    /// No action is currently pending.
    Idle,
}
