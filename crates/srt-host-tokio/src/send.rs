use srt::core::MessageId;

use crate::{Error, HostDriver, Result};

impl<Io> HostDriver<Io> {
    pub fn send_message(&mut self, message: &[u8]) -> Result<MessageId> {
        self.engine.send(message).map_err(Error::from)
    }
}
