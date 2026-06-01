use crate::{Error, ErrorKind, ReceivedMessage, Result, SendFailedEvent};

/// Default received-message queue capacity.
pub const MESSAGE_QUEUE_CAPACITY: usize = 8;
/// Default send-failed queue capacity.
pub const SEND_FAILED_QUEUE_CAPACITY: usize = 8;

/// Fixed storage for adapter output events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventQueues {
    message_queue: [Option<ReceivedMessage>; MESSAGE_QUEUE_CAPACITY],
    message_head: usize,
    message_len: usize,
    send_failed_queue: [Option<SendFailedEvent>; SEND_FAILED_QUEUE_CAPACITY],
    send_failed_head: usize,
    send_failed_len: usize,
}

impl EventQueues {
    /// Creates empty event queues.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            message_queue: [None; MESSAGE_QUEUE_CAPACITY],
            message_head: 0,
            message_len: 0,
            send_failed_queue: [None; SEND_FAILED_QUEUE_CAPACITY],
            send_failed_head: 0,
            send_failed_len: 0,
        }
    }

    /// Pushes one received message.
    pub fn push_message(&mut self, message: ReceivedMessage) -> Result<()> {
        if self.message_len >= MESSAGE_QUEUE_CAPACITY {
            return Err(Error::new(ErrorKind::MessageQueueFull));
        }

        let tail = (self.message_head + self.message_len) % MESSAGE_QUEUE_CAPACITY;
        self.message_queue[tail] = Some(message);
        self.message_len += 1;
        Ok(())
    }

    /// Polls one received message.
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

    /// Pushes one send-failed event.
    pub fn push_send_failed(&mut self, failed: SendFailedEvent) -> Result<()> {
        if self.send_failed_len >= SEND_FAILED_QUEUE_CAPACITY {
            return Err(Error::new(ErrorKind::SendFailedQueueFull));
        }

        let tail = (self.send_failed_head + self.send_failed_len) % SEND_FAILED_QUEUE_CAPACITY;
        self.send_failed_queue[tail] = Some(failed);
        self.send_failed_len += 1;
        Ok(())
    }

    /// Polls one send-failed event.
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
}

impl Default for EventQueues {
    fn default() -> Self {
        Self::new()
    }
}
