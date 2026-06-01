use srt::{Engine, Message, SendFailed};

const MESSAGE_QUEUE_CAPACITY: usize = 8;
const SEND_FAILED_QUEUE_CAPACITY: usize = 8;

/// Drives SRT engine state over an async UART-like byte stream.
#[derive(Debug)]
pub struct UartDriver<Uart> {
    pub(crate) uart: Uart,
    pub(crate) engine: Engine,
    pub(crate) message_queue: [Option<Message>; MESSAGE_QUEUE_CAPACITY],
    pub(crate) message_head: usize,
    pub(crate) message_len: usize,
    pub(crate) send_failed_queue: [Option<SendFailed>; SEND_FAILED_QUEUE_CAPACITY],
    pub(crate) send_failed_head: usize,
    pub(crate) send_failed_len: usize,
}

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver from an existing UART object and SRT engine.
    #[must_use]
    pub const fn new(uart: Uart, engine: Engine) -> Self {
        Self {
            uart,
            engine,
            message_queue: [None; MESSAGE_QUEUE_CAPACITY],
            message_head: 0,
            message_len: 0,
            send_failed_queue: [None; SEND_FAILED_QUEUE_CAPACITY],
            send_failed_head: 0,
            send_failed_len: 0,
        }
    }

    /// Returns a shared reference to the inner engine.
    #[must_use]
    pub const fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns a mutable reference to the inner engine.
    #[must_use]
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Releases the UART and engine.
    #[must_use]
    pub fn into_parts(self) -> (Uart, Engine) {
        (self.uart, self.engine)
    }

    /// Polls one completed incoming message if available.
    #[must_use]
    pub fn poll_message(&mut self) -> Option<Message> {
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
    pub fn poll_send_failed(&mut self) -> Option<SendFailed> {
        if self.send_failed_len == 0 {
            return None;
        }

        let index = self.send_failed_head;
        self.send_failed_head = (self.send_failed_head + 1) % SEND_FAILED_QUEUE_CAPACITY;
        self.send_failed_len -= 1;
        self.send_failed_queue[index].take()
    }

    pub(crate) fn push_message(&mut self, message: Message) -> bool {
        if self.message_len >= MESSAGE_QUEUE_CAPACITY {
            return false;
        }

        let tail = (self.message_head + self.message_len) % MESSAGE_QUEUE_CAPACITY;
        self.message_queue[tail] = Some(message);
        self.message_len += 1;
        true
    }

    pub(crate) fn push_send_failed(&mut self, failed: SendFailed) -> bool {
        if self.send_failed_len >= SEND_FAILED_QUEUE_CAPACITY {
            return false;
        }

        let tail = (self.send_failed_head + self.send_failed_len) % SEND_FAILED_QUEUE_CAPACITY;
        self.send_failed_queue[tail] = Some(failed);
        self.send_failed_len += 1;
        true
    }
}
