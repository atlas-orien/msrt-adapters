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

    /// Dispatches all queued adapter events.
    pub fn dispatch<MessageHandler, SendFailedHandler>(
        &mut self,
        mut handle_message: MessageHandler,
        mut handle_send_failed: SendFailedHandler,
    ) where
        MessageHandler: FnMut(ReceivedMessage),
        SendFailedHandler: FnMut(SendFailedEvent),
    {
        while self.message_len > 0 {
            let index = self.message_head;
            self.message_head = (self.message_head + 1) % MESSAGE_QUEUE_CAPACITY;
            self.message_len -= 1;
            if let Some(message) = self.message_queue[index].take() {
                handle_message(message);
            }
        }

        while self.send_failed_len > 0 {
            let index = self.send_failed_head;
            self.send_failed_head = (self.send_failed_head + 1) % SEND_FAILED_QUEUE_CAPACITY;
            self.send_failed_len -= 1;
            if let Some(failed) = self.send_failed_queue[index].take() {
                handle_send_failed(failed);
            }
        }
    }
}

impl Default for EventQueues {
    fn default() -> Self {
        Self::new()
    }
}
