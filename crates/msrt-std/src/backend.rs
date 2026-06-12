//! Passive std backend adapter.

use std::io::{Read, Write};

use msrt::endpoint::{EngineConfig, PassiveEndpoint, PeerState};

use crate::error::Result;
use crate::event::AdapterEvent;
use crate::io::IoState;

/// Passive std backend adapter.
#[derive(Debug)]
pub struct StdBackend<T> {
    io: T,
    endpoint: PassiveEndpoint,
    state: IoState,
}

impl<T> StdBackend<T>
where
    T: Read + Write,
{
    /// Creates a backend adapter using default MSRT config.
    pub fn new(io: T) -> Self {
        Self::with_config(io, EngineConfig::default())
    }

    /// Creates a backend adapter using `config`.
    pub fn with_config(io: T, config: EngineConfig) -> Self {
        Self {
            io,
            endpoint: PassiveEndpoint::new(config),
            state: IoState::new(),
        }
    }

    /// Returns the endpoint peer state.
    pub fn peer_state(&self) -> PeerState {
        self.endpoint.peer().state()
    }

    /// Returns a shared reference to the owned IO object.
    pub const fn io(&self) -> &T {
        &self.io
    }

    /// Returns a mutable reference to the owned IO object.
    pub fn io_mut(&mut self) -> &mut T {
        &mut self.io
    }

    /// Consumes the adapter and returns the owned IO object.
    pub fn into_inner(self) -> T {
        self.io
    }

    /// Drops the active backend session.
    pub fn disconnect(&mut self) {
        self.endpoint.disconnect();
    }

    /// Queues an application message.
    pub fn send(&mut self, message: &[u8]) -> Result<bool> {
        Ok(self.endpoint.send(message)?.is_some())
    }

    /// Reads currently available stream bytes and feeds them into MSRT.
    ///
    /// This method is best used with nonblocking IO. `WouldBlock` is treated as
    /// no input.
    pub fn receive_available(&mut self) -> Result<usize> {
        self.state.receive_available(&mut self.io, |now_ms, bytes| {
            self.endpoint.receive(now_ms, bytes)
        })
    }

    /// Polls one adapter event and writes any pending transmit bytes.
    pub fn poll(&mut self) -> Result<AdapterEvent> {
        self.state.poll(&mut self.io, |now_ms, tx_buf| {
            self.endpoint.poll(now_ms, tx_buf)
        })
    }

    /// Runs `receive_available` followed by `poll`.
    pub fn tick(&mut self) -> Result<AdapterEvent> {
        let _ = self.receive_available()?;
        self.poll()
    }
}
