use embedded_io_async::{Read, Write};
use srt::{Message, Receive};

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Read + Write,
{
    /// Receives one complete SRT message.
    ///
    /// The method waits for incoming UART bytes, feeds them into the SRT engine,
    /// flushes generated ACK/response writes, and returns when a full message is
    /// available.
    pub async fn receive(&mut self, rx_buf: &mut [u8]) -> Result<Message> {
        loop {
            if let Some(message) = self.pending_message.take() {
                return Ok(message);
            }

            let len = self.uart.read(rx_buf).await.map_err(Error::uart_read_from)?;
            let report = self.engine.receive(&rx_buf[..len]);

            if let Receive::Error(error) = report {
                return Err(Error::from(error));
            }

            self.flush_engine_io().await?;
        }
    }
}
