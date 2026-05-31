use embedded_io_async::{Read, Write};
use srt::core::MessageId;

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Read + Write,
{
    /// Sends one complete SRT message.
    ///
    /// This call is async and non-blocking for protocol logic: it queues into
    /// the SRT engine immediately, then awaits UART writes required to emit wire
    /// packets.
    pub async fn send(&mut self, message: &[u8]) -> Result<MessageId> {
        let message_id = self.engine.send(message).map_err(Error::from)?;
        self.flush_engine_io().await?;
        Ok(message_id)
    }
}
