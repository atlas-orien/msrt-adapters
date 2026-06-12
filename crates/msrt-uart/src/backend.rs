//! Passive UART backend for MCU/no-std use.

use msrt::endpoint::{EndpointPoll, EngineConfig, PassiveEndpoint, PeerState};

use crate::event::UartBackendEvent;
use crate::traits::{UartError, UartIo, UartIoError, UartResult};

const RX_BYTES: usize = 512;
const TX_BYTES: usize = 256;

/// Passive UART backend adapter.
///
/// This type does not require `std` or heap allocation. The caller provides
/// time through `now_ms` arguments, usually from SysTick or an RTOS tick.
#[derive(Debug)]
pub struct UartBackend<T> {
    io: T,
    endpoint: PassiveEndpoint,
    rx_buf: [u8; RX_BYTES],
    tx_buf: [u8; TX_BYTES],
}

impl<T> UartBackend<T>
where
    T: UartIo,
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
            rx_buf: [0; RX_BYTES],
            tx_buf: [0; TX_BYTES],
        }
    }

    /// Returns the endpoint peer state.
    pub fn peer_state(&self) -> PeerState {
        self.endpoint.peer().state()
    }

    /// Returns a shared reference to the owned UART IO object.
    pub const fn io(&self) -> &T {
        &self.io
    }

    /// Returns a mutable reference to the owned UART IO object.
    pub fn io_mut(&mut self) -> &mut T {
        &mut self.io
    }

    /// Consumes the adapter and returns the UART IO object.
    pub fn into_inner(self) -> T {
        self.io
    }

    /// Drops the active backend session.
    pub fn disconnect(&mut self) {
        self.endpoint.disconnect();
    }

    /// Queues an application message.
    pub fn send(&mut self, message: &[u8]) -> UartResult<bool, T::Error> {
        self.endpoint
            .send(message)
            .map(|id| id.is_some())
            .map_err(UartError::Protocol)
    }

    /// Reads currently available UART bytes and feeds them into MSRT.
    pub fn receive_available(&mut self, now_ms: u64) -> UartResult<usize, T::Error> {
        let mut total = 0;
        loop {
            match self.io.read(&mut self.rx_buf) {
                Ok(0) => return Ok(total),
                Ok(n) => {
                    total += n;
                    let _ = self.endpoint.receive(now_ms, &self.rx_buf[..n]);
                    if n < self.rx_buf.len() {
                        return Ok(total);
                    }
                }
                Err(UartIoError::WouldBlock) => return Ok(total),
                Err(UartIoError::Interrupted) => continue,
                Err(error) => return Err(UartError::Io(error)),
            }
        }
    }

    /// Polls one backend event and writes pending UART bytes.
    pub fn poll(&mut self, now_ms: u64) -> UartResult<UartBackendEvent, T::Error> {
        loop {
            match self
                .endpoint
                .poll(now_ms, &mut self.tx_buf)
                .map_err(UartError::Protocol)?
            {
                EndpointPoll::Transmit { bytes, .. } => {
                    self.io.write_all(bytes).map_err(UartError::Io)?;
                }
                EndpointPoll::Message(message) => return Ok(UartBackendEvent::Message(message)),
                EndpointPoll::SendFailed(failed) => {
                    return Ok(UartBackendEvent::SendFailed(failed));
                }
                EndpointPoll::Idle => return Ok(UartBackendEvent::Idle),
            }
        }
    }

    /// Runs `receive_available` followed by `poll`.
    pub fn tick(&mut self, now_ms: u64) -> UartResult<UartBackendEvent, T::Error> {
        let _ = self.receive_available(now_ms)?;
        self.poll(now_ms)
    }
}
