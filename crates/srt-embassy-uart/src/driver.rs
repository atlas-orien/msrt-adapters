use srt::{Engine, Message};

/// Drives an SRT engine with an async UART-like byte stream.
///
/// The adapter owns both UART and engine state and exposes a `tokio`-like API:
/// - `send(message).await`
/// - `receive(rx_buf).await`
#[derive(Debug)]
pub struct UartDriver<Uart> {
    pub(crate) uart: Uart,
    pub(crate) engine: Engine,
    pub(crate) pending_message: Option<Message>,
}

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver from an existing UART object and SRT engine.
    #[must_use]
    pub const fn new(uart: Uart, engine: Engine) -> Self {
        Self {
            uart,
            engine,
            pending_message: None,
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
}
