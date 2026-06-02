/// A fully received message surfaced by an adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceivedMessage {
    channel_id: u8,
    message_id: u32,
    bytes: [u8; msrt::MAX_MESSAGE_BYTES],
    len: usize,
}

impl ReceivedMessage {
    /// Creates a received message from an MSRT engine event.
    #[must_use]
    pub const fn from_srt(message: msrt::Message) -> Self {
        Self {
            channel_id: message.channel_id.get(),
            message_id: message.message_id.get(),
            bytes: message.bytes,
            len: message.len,
        }
    }

    /// Returns the raw channel identifier as `u16` for ergonomic matching.
    #[must_use]
    pub const fn channel_id(self) -> u16 {
        self.channel_id as u16
    }

    /// Returns the raw channel identifier.
    #[must_use]
    pub const fn channel_id_u8(self) -> u8 {
        self.channel_id
    }

    /// Returns the message identifier.
    #[must_use]
    pub const fn message_id(self) -> u32 {
        self.message_id
    }

    /// Returns valid message bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.bytes.split_at(self.len).0
    }
}

/// Reason a reliable send failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendFailedReason {
    /// Retry limit was reached before acknowledgement.
    RetryLimitReached,
}

/// A reliable-send failure surfaced by an adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendFailedEvent {
    channel_id: u8,
    message_id: u32,
    reason: SendFailedReason,
}

impl SendFailedEvent {
    /// Creates a send-failed event from an MSRT engine event.
    #[must_use]
    pub const fn from_srt(failed: msrt::SendFailed) -> Self {
        Self {
            channel_id: failed.channel_id.get(),
            message_id: failed.message_id.get(),
            reason: SendFailedReason::RetryLimitReached,
        }
    }

    /// Returns the raw channel identifier as `u16` for ergonomic matching.
    #[must_use]
    pub const fn channel_id(self) -> u16 {
        self.channel_id as u16
    }

    /// Returns the raw channel identifier.
    #[must_use]
    pub const fn channel_id_u8(self) -> u8 {
        self.channel_id
    }

    /// Returns the message identifier.
    #[must_use]
    pub const fn message_id(self) -> u32 {
        self.message_id
    }

    /// Returns the failure reason.
    #[must_use]
    pub const fn reason(self) -> SendFailedReason {
        self.reason
    }
}
