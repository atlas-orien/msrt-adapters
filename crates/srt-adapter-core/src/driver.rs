use core::ops::{Deref, DerefMut};

use crate::{AdapterCore, ReceivedMessage, Result, SendFailedEvent};

/// Generic adapter driver that owns one platform I/O object and shared SRT state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterDriver<Io> {
    io: Io,
    core: AdapterCore,
}

impl<Io> AdapterDriver<Io> {
    /// Creates a driver with default SRT adapter core.
    #[must_use]
    pub fn new(io: Io) -> Self {
        Self {
            io,
            core: AdapterCore::new(),
        }
    }

    /// Releases the platform I/O object.
    #[must_use]
    pub fn into_io(self) -> Io {
        self.io
    }

    /// Returns mutable references to I/O and adapter core.
    pub fn parts_mut(&mut self) -> (&mut Io, &mut AdapterCore) {
        (&mut self.io, &mut self.core)
    }

    /// Submits one complete application message.
    pub fn send_message(&mut self, message: &[u8]) -> Result<()> {
        self.core.send_message(message)
    }

    /// Sends one debug log message.
    pub fn debug(&mut self, message: &[u8]) -> Result<()> {
        self.core.debug(message)
    }

    /// Polls one completed incoming message if available.
    #[must_use]
    pub fn poll_message(&mut self) -> Option<ReceivedMessage> {
        self.core.poll_message()
    }

    /// Polls one reliable-send failure event if available.
    #[must_use]
    pub fn poll_send_failed(&mut self) -> Option<SendFailedEvent> {
        self.core.poll_send_failed()
    }
}

/// Wrapper helper for platform-specific driver newtypes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriverWrapper<Io>(pub AdapterDriver<Io>);

impl<Io> Deref for DriverWrapper<Io> {
    type Target = AdapterDriver<Io>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Io> DerefMut for DriverWrapper<Io> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
