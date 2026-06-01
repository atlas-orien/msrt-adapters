use embedded_io_async::{Read, Write};

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Read + Write,
{
    /// Advances protocol state once.
    ///
    /// One call performs a bounded step:
    /// tick time, drain engine events, then read UART bytes and feed engine.
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.core.begin_poll(now_ms);
        self.drain_writes().await?;

        let len = self.uart.read(rx_buf).await.map_err(Error::embedded_io_read)?;
        self.core.read_completed(&rx_buf[..len])?;
        self.drain_writes().await
    }

    async fn drain_writes(&mut self) -> Result<()> {
        while let Some(write) = self.core.poll_write()? {
            self.uart
                .write_all(write.as_bytes())
                .await
                .map_err(Error::embedded_io_write)?;
            self.uart.flush().await.map_err(Error::embedded_io_flush)?;
        }

        Ok(())
    }
}
