use crate::{HostDriver, Result};

impl<Io> HostDriver<Io> {
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.core.send_message(message)
    }

    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.core.debug(message)
    }
}
