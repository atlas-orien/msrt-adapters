use crate::{Error, ErrorKind, ReceivedMessage, Result, SendFailedEvent};

/// Handles pending adapter core events.
pub trait PendingEventHandler {
    /// Handles one complete received message.
    fn handle_message(&mut self, message: ReceivedMessage);

    /// Handles one reliable-send failure event.
    fn handle_send_failed(&mut self, failed: SendFailedEvent);
}

/// Default received-message event capacity.
pub const MESSAGE_EVENT_CAPACITY: usize = 8;
/// Default send-failed event capacity.
pub const SEND_FAILED_EVENT_CAPACITY: usize = 8;

/// Fixed storage for adapter output events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingEvents {
    message_events: [Option<ReceivedMessage>; MESSAGE_EVENT_CAPACITY],
    message_head: usize,
    message_len: usize,
    send_failed_events: [Option<SendFailedEvent>; SEND_FAILED_EVENT_CAPACITY],
    send_failed_head: usize,
    send_failed_len: usize,
}

impl PendingEvents {
    /// Creates empty pending events.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            message_events: [None; MESSAGE_EVENT_CAPACITY],
            message_head: 0,
            message_len: 0,
            send_failed_events: [None; SEND_FAILED_EVENT_CAPACITY],
            send_failed_head: 0,
            send_failed_len: 0,
        }
    }

    /// Pushes one received message.
    pub fn push_message(&mut self, message: ReceivedMessage) -> Result<()> {
        if self.message_len >= MESSAGE_EVENT_CAPACITY {
            return Err(Error::new(ErrorKind::MessageEventsFull));
        }

        let tail = (self.message_head + self.message_len) % MESSAGE_EVENT_CAPACITY;
        self.message_events[tail] = Some(message);
        self.message_len += 1;
        Ok(())
    }

    /// Pushes one send-failed event.
    pub fn push_send_failed(&mut self, failed: SendFailedEvent) -> Result<()> {
        if self.send_failed_len >= SEND_FAILED_EVENT_CAPACITY {
            return Err(Error::new(ErrorKind::SendFailedEventsFull));
        }

        let tail = (self.send_failed_head + self.send_failed_len) % SEND_FAILED_EVENT_CAPACITY;
        self.send_failed_events[tail] = Some(failed);
        self.send_failed_len += 1;
        Ok(())
    }

    /// Dispatches all pending adapter events.
    pub fn dispatch<Handler>(&mut self, handler: &mut Handler)
    where
        Handler: PendingEventHandler,
    {
        while self.message_len > 0 {
            let index = self.message_head;
            self.message_head = (self.message_head + 1) % MESSAGE_EVENT_CAPACITY;
            self.message_len -= 1;
            if let Some(message) = self.message_events[index].take() {
                handler.handle_message(message);
            }
        }

        while self.send_failed_len > 0 {
            let index = self.send_failed_head;
            self.send_failed_head = (self.send_failed_head + 1) % SEND_FAILED_EVENT_CAPACITY;
            self.send_failed_len -= 1;
            if let Some(failed) = self.send_failed_events[index].take() {
                handler.handle_send_failed(failed);
            }
        }
    }
}

impl Default for PendingEvents {
    fn default() -> Self {
        Self::new()
    }
}
