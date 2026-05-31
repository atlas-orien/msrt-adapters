use embedded_io_async::Write;
use srt::Event;

use crate::{Error, Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Write,
{
    pub(crate) async fn flush_engine_io(&mut self) -> Result<()> {
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
                    return Err(Error::from(failed));
                }
            }
        }

        Ok(())
    }
}
