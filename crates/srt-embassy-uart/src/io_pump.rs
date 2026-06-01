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
        self.parts_mut().1.begin_poll(now_ms);
        self.drain_writes().await?;

        let (uart, core) = self.parts_mut();
        let len = uart.read(rx_buf).await.map_err(Error::embedded_io_read)?;
        core.read_completed(&rx_buf[..len])?;
        self.drain_writes().await
    }

    async fn drain_writes(&mut self) -> Result<()> {
        loop {
            let write = {
                let (_, core) = self.parts_mut();
                core.poll_write()?
            };
            let Some(write) = write else {
                return Ok(());
            };

            let (uart, _) = self.parts_mut();
            uart.write_all(write.as_bytes())
                .await
                .map_err(Error::embedded_io_write)?;
            uart.flush().await.map_err(Error::embedded_io_flush)?;
        }
    }
}
