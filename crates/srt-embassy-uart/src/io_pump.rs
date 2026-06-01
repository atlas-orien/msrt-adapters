use embedded_io_async::{Read, Write};
use srt::{Event, Receive};

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Read + Write,
{
    /// Advances protocol state once.
    ///
    /// One call performs a bounded step:
    /// read UART bytes, feed engine receive, tick time, then drain engine events.
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        let len = self.uart.read(rx_buf).await.map_err(Error::uart_read_from)?;

        if len > 0 {
            let report = self.engine.receive(&rx_buf[..len]);
            if let Receive::Error(error) = report {
                return Err(Error::from(error));
            }
        }

        self.engine.tick(now_ms);
        self.drain_engine_events().await
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
                    self.pending_message = Some(message);
                }
                Event::SendFailed(failed) => {
                    self.pending_send_failed = Some(failed);
                }
            }
        }

        Ok(())
    }
}
