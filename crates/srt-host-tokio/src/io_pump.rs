use srt::{Event, Receive};
use tokio::time::{Duration, timeout};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Error, ErrorKind, HostDriver, Result};

impl<Io> HostDriver<Io>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.engine.tick(now_ms);
        self.drain_engine_events().await?;

        let len = match timeout(Duration::from_millis(1), self.io.read(rx_buf)).await {
            Ok(result) => result.map_err(|_| Error::new(ErrorKind::IoRead))?,
            Err(_) => 0,
        };

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
                    self.io
                        .write_all(write.as_bytes())
                        .await
                        .map_err(|_| Error::new(ErrorKind::IoWrite))?;
                    self.io
                        .flush()
                        .await
                        .map_err(|_| Error::new(ErrorKind::IoFlush))?;
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
