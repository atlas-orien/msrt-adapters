use srt_adapter_core::{AdapterCore, ReceivedMessage, SendFailedEvent};

#[derive(Debug)]
pub struct HostDriver<Io> {
    pub(crate) io: Io,
    pub(crate) core: AdapterCore,
}

impl<Io> HostDriver<Io> {
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

    #[must_use]
    pub fn poll_message(&mut self) -> Option<ReceivedMessage> {
        self.core.poll_message()
    }

    #[must_use]
    pub fn poll_send_failed(&mut self) -> Option<SendFailedEvent> {
        self.core.poll_send_failed()
    }
}
