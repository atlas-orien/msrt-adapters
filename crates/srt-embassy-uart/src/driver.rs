use srt_adapter_core::{AdapterCore, ReceivedMessage, SendFailedEvent};

/// Drives SRT engine state over an async UART-like byte stream.
#[derive(Debug)]
pub struct UartDriver<Uart> {
    pub(crate) uart: Uart,
    pub(crate) core: AdapterCore,
}

impl<Uart> UartDriver<Uart> {
    /// Creates a UART driver with default SRT engine config.
    #[must_use]
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            core: AdapterCore::new(),
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
        self.core.poll_message()
    }

    /// Polls one reliable-send failure event if available.
    #[must_use]
    pub fn poll_send_failed(&mut self) -> Option<SendFailedEvent> {
        self.core.poll_send_failed()
    }
}
