use embedded_io_async::{Read, Write};
use srt::{Event, Receive};

use crate::{Error, ErrorKind, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Read + Write,
{
    /// Advances protocol state once.
    ///
    /// One call performs a bounded step:
    /// tick time, drain engine events, then read UART bytes and feed engine.
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.engine.tick(now_ms);
        self.drain_engine_events().await?;

        let len = self.uart.read(rx_buf).await.map_err(Error::uart_read_from)?;

        if len > 0 {
            let report = self.engine.receive(&rx_buf[..len]);
            if let Receive::Error(error) = report {
                return Err(Error::from(error));
            }
            self.drain_engine_events().await?;
        }

        Ok(())
    }

    async fn drain_engine_events(&mut self) -> Result<()> {
        while let Some(event) = self.engine.poll_event() {
            match event {
                Event::Write(write) => {
                    self.uart
                        .write_all(write.as_bytes())
                        .await
                        .map_err(Error::uart_write_from)?;
                    self.uart.flush().await.map_err(Error::uart_flush_from)?;
                }
                Event::Message(message) => {
                    if !self.push_message(message) {
                        return Err(Error::new(ErrorKind::MessageQueueFull));
                    }
                }
                Event::SendFailed(failed) => {
                    if !self.push_send_failed(failed) {
                        return Err(Error::new(ErrorKind::SendFailedQueueFull));
                    }
                }
            }
        }

        Ok(())
    }
}
