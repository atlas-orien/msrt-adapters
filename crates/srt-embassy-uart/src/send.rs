use srt::core::MessageId;

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart> {
    /// Submits one complete SRT message into the protocol engine.
    ///
    /// This does not mean reliable delivery is complete. Delivery progresses in
    /// later `poll_once` calls.
    pub fn send_message(&mut self, message: &[u8]) -> Result<MessageId> {
        self.engine.send(message).map_err(Error::from)
    }
}
