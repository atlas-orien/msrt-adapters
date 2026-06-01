use srt::Engine;

const MESSAGE_QUEUE_CAPACITY: usize = 8;
const SEND_FAILED_QUEUE_CAPACITY: usize = 8;

/// A fully received message surfaced by this adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceivedMessage {
    channel_id: u16,
    message_id: u32,
    bytes: [u8; srt::MAX_MESSAGE_BYTES],
    len: usize,
}

impl ReceivedMessage {
    #[must_use]
    pub const fn channel_id(self) -> u16 {
        self.channel_id
    }

    #[must_use]
    pub const fn message_id(self) -> u32 {
        self.message_id
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.bytes.split_at(self.len).0
    }
}

/// Reason a reliable send failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendFailedReason {
    RetryLimitReached,
}

/// A reliable-send failure surfaced by this adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendFailedEvent {
    channel_id: u16,
    message_id: u32,
    reason: SendFailedReason,
}

impl SendFailedEvent {
    #[must_use]
    pub const fn channel_id(self) -> u16 {
        self.channel_id
    }

    #[must_use]
    pub const fn message_id(self) -> u32 {
        self.message_id
    }

    #[must_use]
    pub const fn reason(self) -> SendFailedReason {
        self.reason
    }
}

/// Drives SRT engine state over an async UART-like byte stream.
#[derive(Debug)]
pub struct UartDriver<Uart> {
    pub(crate) uart: Uart,
    pub(crate) engine: Engine,
    pub(crate) message_queue: [Option<ReceivedMessage>; MESSAGE_QUEUE_CAPACITY],
    pub(crate) message_head: usize,
    pub(crate) message_len: usize,
    pub(crate) send_failed_queue: [Option<SendFailedEvent>; SEND_FAILED_QUEUE_CAPACITY],
    pub(crate) send_failed_head: usize,
    pub(crate) send_failed_len: usize,
}

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver with default SRT engine config.
    #[must_use]
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            engine: Engine::new(srt::Config::default()),
            message_queue: [None; MESSAGE_QUEUE_CAPACITY],
            message_head: 0,
            message_len: 0,
            send_failed_queue: [None; SEND_FAILED_QUEUE_CAPACITY],
            send_failed_head: 0,
            send_failed_len: 0,
        }
    }

    /// Releases only the UART object.
    #[must_use]
    pub fn into_uart(self) -> Uart {
        self.uart
    }

    /// Polls one completed incoming message if available.
    #[must_use]
    pub fn poll_message(&mut self) -> Option<ReceivedMessage> {
        if self.message_len == 0 {
            return None;
        }

        let index = self.message_head;
        self.message_head = (self.message_head + 1) % MESSAGE_QUEUE_CAPACITY;
        self.message_len -= 1;
        self.message_queue[index].take()
    }

    /// Polls one reliable-send failure event if available.
    #[must_use]
    pub fn poll_send_failed(&mut self) -> Option<SendFailedEvent> {
        if self.send_failed_len == 0 {
            return None;
        }

        let index = self.send_failed_head;
        self.send_failed_head = (self.send_failed_head + 1) % SEND_FAILED_QUEUE_CAPACITY;
        self.send_failed_len -= 1;
        self.send_failed_queue[index].take()
    }

    pub(crate) fn push_message(&mut self, message: srt::Message) -> bool {
        if self.message_len >= MESSAGE_QUEUE_CAPACITY {
            return false;
        }

        let tail = (self.message_head + self.message_len) % MESSAGE_QUEUE_CAPACITY;
        self.message_queue[tail] = Some(ReceivedMessage {
            channel_id: message.channel_id.get(),
            message_id: message.message_id.get(),
            bytes: message.bytes,
            len: message.len,
        });
        self.message_len += 1;
        true
    }

    pub(crate) fn push_send_failed(&mut self, failed: srt::SendFailed) -> bool {
        if self.send_failed_len >= SEND_FAILED_QUEUE_CAPACITY {
            return false;
        }

        let tail = (self.send_failed_head + self.send_failed_len) % SEND_FAILED_QUEUE_CAPACITY;
        self.send_failed_queue[tail] = Some(SendFailedEvent {
            channel_id: failed.channel_id.get(),
            message_id: failed.message_id.get(),
            reason: SendFailedReason::RetryLimitReached,
        });
        self.send_failed_len += 1;
        true
    }
}
