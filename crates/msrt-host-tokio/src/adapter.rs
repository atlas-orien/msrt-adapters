use msrt_adapter_core::{
    AdapterCore, PendingEventHandler, ReceivedMessage, Result, SendFailedEvent,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::{Duration, timeout};

use crate::{Error, ErrorKind};

/// Error surfaced by the managed host adapter task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostTaskError {
    /// The adapter failed while polling the host I/O or MSRT core.
    Adapter(Error),
    /// A reliable send reached its retry limit.
    SendFailed(SendFailedEvent),
}

/// Handles events produced by a host adapter task.
pub trait HostEventHandler {
    fn handle_message(&mut self, message: ReceivedMessage);

    fn handle_error(&mut self, error: HostTaskError);
}

struct CoreEventHandler<'a, Handler> {
    handler: &'a mut Handler,
}

impl<Handler> PendingEventHandler for CoreEventHandler<'_, Handler>
where
    Handler: HostEventHandler,
{
    fn handle_message(&mut self, message: ReceivedMessage) {
        self.handler.handle_message(message);
    }

    fn handle_send_failed(&mut self, failed: SendFailedEvent) {
        self.handler.handle_error(HostTaskError::SendFailed(failed));
    }
}

/// Adapts MSRT protocol state to a Tokio async byte stream.
#[derive(Debug)]
pub struct HostAdapter<Io> {
    pub(crate) io: Io,
    pub(crate) core: AdapterCore,
}

impl<Io> HostAdapter<Io> {
    #[must_use]
    pub fn new(io: Io) -> Self {
        Self {
            io,
            core: AdapterCore::new(),
        }
    }

    #[must_use]
    pub fn into_io(self) -> Io {
        self.io
    }

    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.core.send_message(message)
    }

    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.core.debug(message)
    }
}

impl<Io> HostAdapter<Io>
where
    Io: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.core.tick(now_ms);
        self.drain_writes().await?;

        let len = match timeout(Duration::from_millis(1), self.io.read(rx_buf)).await {
            Ok(result) => result.map_err(|_| Error::new(ErrorKind::IoRead))?,
            Err(_) => 0,
        };

        self.core.read_completed(&rx_buf[..len])?;
        self.drain_writes().await
    }

    pub async fn poll_once_dispatch<Handler>(
        &mut self,
        now_ms: u64,
        rx_buf: &mut [u8],
        handler: &mut Handler,
    ) where
        Handler: HostEventHandler,
    {
        if let Err(error) = self.poll_once(now_ms, rx_buf).await {
            handler.handle_error(HostTaskError::Adapter(error));
        }

        self.core.dispatch_events(&mut CoreEventHandler { handler });
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
