use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart> {
    /// Submits one complete message into the protocol engine.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.engine.send(message).map_err(Error::from)?;
        Ok(())
    }
}
