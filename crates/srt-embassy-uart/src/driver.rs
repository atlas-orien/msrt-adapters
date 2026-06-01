use srt::{Engine, Message, SendFailed};

/// Drives SRT engine state over an async UART-like byte stream.
#[derive(Debug)]
pub struct UartDriver<Uart> {
    pub(crate) uart: Uart,
    pub(crate) engine: Engine,
    pub(crate) pending_message: Option<Message>,
    pub(crate) pending_send_failed: Option<SendFailed>,
}

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver from an existing UART object and SRT engine.
    #[must_use]
    pub const fn new(uart: Uart, engine: Engine) -> Self {
        Self {
            uart,
            engine,
            pending_message: None,
            pending_send_failed: None,
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
        self.pending_message.take()
    }

    /// Polls one reliable-send failure event if available.
    #[must_use]
    pub fn poll_send_failed(&mut self) -> Option<SendFailed> {
        self.pending_send_failed.take()
    }
}
