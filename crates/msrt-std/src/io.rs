//! Shared std byte-stream IO driver.

use std::io::{ErrorKind, Read, Write};
use std::time::Instant;

use msrt::endpoint::{EndpointPoll, ReceiveReport};

use crate::error::Result;
use crate::event::AdapterEvent;

pub(crate) const DEFAULT_RX_BYTES: usize = 512;
pub(crate) const DEFAULT_TX_BYTES: usize = 256;

pub(crate) struct IoState {
    start: Instant,
    rx_buf: [u8; DEFAULT_RX_BYTES],
    tx_buf: [u8; DEFAULT_TX_BYTES],
}

impl IoState {
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
            rx_buf: [0; DEFAULT_RX_BYTES],
            tx_buf: [0; DEFAULT_TX_BYTES],
        }
    }

    pub(crate) fn now_ms(&self) -> u64 {
        self.start
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    pub(crate) fn receive_available<T, F>(&mut self, io: &mut T, mut receive: F) -> Result<usize>
    where
        T: Read,
        F: FnMut(u64, &[u8]) -> ReceiveReport,
    {
        let mut total = 0;
        loop {
            match io.read(&mut self.rx_buf) {
                Ok(0) => return Ok(total),
                Ok(n) => {
                    total += n;
                    let now_ms = self.now_ms();
                    let bytes = &self.rx_buf[..n];
                    let _ = receive(now_ms, bytes);
                    if n < self.rx_buf.len() {
                        return Ok(total);
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(total),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    pub(crate) fn poll<T, F>(&mut self, io: &mut T, mut poll: F) -> Result<AdapterEvent>
    where
        T: Write,
        F: FnMut(u64, &mut [u8]) -> msrt::error::Result<EndpointPoll<'_>>,
    {
        let now_ms = self.now_ms();
        loop {
            match poll(now_ms, &mut self.tx_buf)? {
                EndpointPoll::Transmit { bytes, .. } => io.write_all(bytes)?,
                EndpointPoll::Message(message) => return Ok(AdapterEvent::Message(message)),
                EndpointPoll::SendFailed(failed) => return Ok(AdapterEvent::SendFailed(failed)),
                EndpointPoll::Idle => return Ok(AdapterEvent::Idle),
            }
        }
    }
}

impl core::fmt::Debug for IoState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("IoState")
            .field("rx_buf_len", &self.rx_buf.len())
            .field("tx_buf_len", &self.tx_buf.len())
            .finish_non_exhaustive()
    }
}
