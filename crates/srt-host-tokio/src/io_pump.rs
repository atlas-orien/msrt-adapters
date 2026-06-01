use tokio::time::{Duration, timeout};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, ErrorKind, HostDriver, Result};

impl<Io> HostDriver<Io>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.core.begin_poll(now_ms);
        self.drain_writes().await?;

        let len = match timeout(Duration::from_millis(1), self.io.read(rx_buf)).await {
            Ok(result) => result.map_err(|_| Error::new(ErrorKind::IoRead))?,
            Err(_) => 0,
        };

        self.core.read_completed(&rx_buf[..len])?;
        self.drain_writes().await
    }

    async fn drain_writes(&mut self) -> Result<()> {
        while let Some(write) = self.core.poll_write()? {
            self.io
                .write_all(write.as_bytes())
                .await
                .map_err(|_| Error::new(ErrorKind::IoWrite))?;
            self.io
                .flush()
                .await
                .map_err(|_| Error::new(ErrorKind::IoFlush))?;
        }

        Ok(())
    }
}
