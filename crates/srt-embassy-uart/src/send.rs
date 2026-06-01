use crate::{Error, Result, UartDriver};
use srt::ChannelId;

impl<Uart> UartDriver<Uart> {
    /// Submits one complete message into the protocol engine.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.engine.send(message).map_err(Error::from)?;
        Ok(())
    }

    /// Sends one debug log message on the SRT log channel.
    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.engine
            .send_on(ChannelId::LOG, message)
            .map_err(Error::from)?;
        Ok(())
    }
}
