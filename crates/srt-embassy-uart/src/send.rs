use crate::{Result, UartDriver};

impl<Uart> UartDriver<Uart> {
    /// Submits one complete message into the protocol engine.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.core.send_message(message)
    }

    /// Sends one debug log message on the SRT log channel.
    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.core.debug(message)
    }
}
