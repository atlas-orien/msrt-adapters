use embedded_io_async::{Read, Write};
use srt_adapter_core::{AdapterCore, ReceivedMessage, Result, SendFailedEvent};

use crate::Error;

/// Error surfaced by the managed UART adapter task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartTaskError {
    /// The adapter failed while polling the UART or SRT core.
    Adapter(Error),
    /// A reliable send reached its retry limit.
    SendFailed(SendFailedEvent),
}

/// Adapts SRT protocol state to an async UART-like byte stream.
#[derive(Debug)]
pub struct UartAdapter<Uart> {
    pub(crate) uart: Uart,
    pub(crate) core: AdapterCore,
}

impl<Uart> UartAdapter<Uart> {
    /// Creates a UART adapter with default SRT engine config.
    #[must_use]
    pub fn new(uart: Uart) -> Self {
        Self {
            uart,
            core: AdapterCore::new(),
        }
    }

    /// Releases only the UART object.
    #[must_use]
    pub fn into_uart(self) -> Uart {
        self.uart
    }

    /// Submits one complete application message.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.core.send_message(message)
    }

    /// Sends one debug log message on the SRT log channel.
    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.core.debug(message)
    }
}

impl<Uart> UartAdapter<Uart>
where
    Uart: Read + Write,
{
    /// Advances protocol state once.
    ///
    /// One call performs a bounded step:
    /// tick time, drain engine events, then read UART bytes and feed engine.
    pub async fn poll_once(&mut self, now_ms: u64, rx_buf: &mut [u8]) -> Result<()> {
        self.core.tick(now_ms);
        self.drain_writes().await?;

        let len = self
            .uart
            .read(rx_buf)
            .await
            .map_err(Error::embedded_io_read)?;
        self.core.read_completed(&rx_buf[..len])?;
        self.drain_writes().await
    }

    /// Advances protocol state once and dispatches resulting events to handlers.
    pub async fn poll_once_dispatch<MessageHandler, ErrorHandler>(
        &mut self,
        now_ms: u64,
        rx_buf: &mut [u8],
        handle_message: MessageHandler,
        mut handle_error: ErrorHandler,
    ) where
        MessageHandler: FnMut(ReceivedMessage),
        ErrorHandler: FnMut(UartTaskError),
    {
        if let Err(error) = self.poll_once(now_ms, rx_buf).await {
            handle_error(UartTaskError::Adapter(error));
        }

        self.core.dispatch_events(handle_message, |failed| {
            handle_error(UartTaskError::SendFailed(failed));
        });
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
