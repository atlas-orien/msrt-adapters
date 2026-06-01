use crate::{Error, EventQueues, ReceivedMessage, Result, SendFailedEvent};

/// Platform-independent SRT adapter state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterCore {
    engine: srt::Engine,
    events: EventQueues,
}

impl AdapterCore {
    /// Creates adapter core with default SRT engine config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: srt::Engine::new(srt::Config::default()),
            events: EventQueues::new(),
        }
    }

    /// Submits one complete application message.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.engine.send(message).map_err(Error::from)?;
        Ok(())
    }

    /// Submits one debug log message on the SRT log channel.
    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.engine
            .send_on(srt::ChannelId::LOG, message)
            .map_err(Error::from)?;
        Ok(())
    }

    /// Feeds already-received wire bytes into SRT.
    pub fn receive(&mut self, bytes: &[u8]) -> Result<()> {
        if let srt::Receive::Error(error) = self.engine.receive(bytes) {
            return Err(Error::from(error));
        }
        Ok(())
    }

    /// Advances time-driven SRT protocol work.
    pub fn tick(&mut self, now_ms: u64) {
        self.engine.tick(now_ms);
    }

    /// Dispatches all completed adapter events to handlers.
    pub fn dispatch_events<MessageHandler, SendFailedHandler>(
        &mut self,
        handle_message: MessageHandler,
        handle_send_failed: SendFailedHandler,
    ) where
        MessageHandler: FnMut(ReceivedMessage),
        SendFailedHandler: FnMut(SendFailedEvent),
    {
        self.events.dispatch(handle_message, handle_send_failed);
    }

    /// Drains engine events, calling `write` for each wire write event.
    pub async fn drain_writes<F, Fut>(&mut self, mut write: F) -> Result<()>
    where
        F: FnMut(srt::Write) -> Fut,
        Fut: core::future::Future<Output = Result<()>>,
    {
        while let Some(event) = self.engine.poll_event() {
            match event {
                srt::Event::Write(write_event) => write(write_event).await?,
                srt::Event::Message(message) => self
                    .events
                    .push_message(ReceivedMessage::from_srt(message))?,
                srt::Event::SendFailed(failed) => self
                    .events
                    .push_send_failed(SendFailedEvent::from_srt(failed))?,
            }
        }

        Ok(())
    }

    /// Polls the next wire write event, queueing non-write events internally.
    pub fn poll_write(&mut self) -> Result<Option<srt::Write>> {
        while let Some(event) = self.engine.poll_event() {
            match event {
                srt::Event::Write(write) => return Ok(Some(write)),
                srt::Event::Message(message) => self
                    .events
                    .push_message(ReceivedMessage::from_srt(message))?,
                srt::Event::SendFailed(failed) => self
                    .events
                    .push_send_failed(SendFailedEvent::from_srt(failed))?,
            }
        }

        Ok(None)
    }

    /// Advances adapter state once using platform-provided I/O operations.
    pub async fn poll_once<R, RFut, W, WFut>(
        &mut self,
        now_ms: u64,
        rx_buf: &mut [u8],
        mut read: R,
        mut write: W,
    ) -> Result<()>
    where
        R: FnMut(&mut [u8]) -> RFut,
        RFut: core::future::Future<Output = Result<usize>>,
        W: FnMut(srt::Write) -> WFut,
        WFut: core::future::Future<Output = Result<()>>,
    {
        self.tick(now_ms);
        self.drain_platform_writes(&mut write).await?;

        let len = read(rx_buf).await?;
        if len > 0 {
            self.receive(&rx_buf[..len])?;
            self.drain_platform_writes(&mut write).await?;
        }

        Ok(())
    }

    async fn drain_platform_writes<W, WFut>(&mut self, write: &mut W) -> Result<()>
    where
        W: FnMut(srt::Write) -> WFut,
        WFut: core::future::Future<Output = Result<()>>,
    {
        while let Some(write_event) = self.poll_write()? {
            write(write_event).await?;
        }

        Ok(())
    }

    /// Completes the read part of one poll step.
    pub fn read_completed(&mut self, bytes: &[u8]) -> Result<()> {
        if !bytes.is_empty() {
            self.receive(bytes)?;
        }

        Ok(())
    }
}

impl Default for AdapterCore {
    fn default() -> Self {
        Self::new()
    }
}
