use embedded_io_async::Write;

use crate::{Result, UartDriver};

impl<Uart> UartDriver<Uart>
where
    Uart: Write,
{
    /// Advances protocol timers and flushes generated writes.
    pub async fn tick(&mut self, now_ms: u64) -> Result<()> {
        self.engine.tick(now_ms);
        self.flush_engine_io().await
    }
}
