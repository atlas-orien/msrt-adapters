use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::{Duration, timeout};

use crate::{Error, ErrorKind, HostDriver, Result};

impl<Io> HostDriver<Io>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.parts_mut().1.begin_poll(now_ms);
        self.drain_writes().await?;

        let (io, core) = self.parts_mut();
        let len = match timeout(Duration::from_millis(1), io.read(rx_buf)).await {
            Ok(result) => result.map_err(|_| Error::new(ErrorKind::IoRead))?,
            Err(_) => 0,
        };

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

            let (io, _) = self.parts_mut();
            io.write_all(write.as_bytes())
                .await
                .map_err(|_| Error::new(ErrorKind::IoWrite))?;
            io.flush()
                .await
                .map_err(|_| Error::new(ErrorKind::IoFlush))?;
        }
    }
}
