//! Tokio host UART frontend.

use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::time::Instant;

use msrt::endpoint::{ClientEndpoint, EndpointPoll, EngineConfig, PeerState};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use crate::event::UartFrontendEvent;
use crate::tokio_error::{TokioError, TokioResult};

const RX_BYTES: usize = 512;
const TX_BYTES: usize = 256;

/// Tokio host UART frontend adapter.
///
/// `T` can be a Tokio serial stream, TCP stream used as a UART-like byte stream,
/// or any async byte stream implementing `AsyncRead + AsyncWrite + Unpin`.
#[derive(Debug)]
pub struct TokioUartFrontend<T> {
    io: T,
    endpoint: ClientEndpoint,
    start: Instant,
    rx_buf: [u8; RX_BYTES],
    tx_buf: [u8; TX_BYTES],
}

impl<T> TokioUartFrontend<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a frontend adapter using default MSRT config.
    pub fn new(io: T) -> Self {
        Self::with_config(io, EngineConfig::default())
    }

    /// Creates a frontend adapter using `config`.
    pub fn with_config(io: T, config: EngineConfig) -> Self {
        Self {
            io,
            endpoint: ClientEndpoint::new(config),
            start: Instant::now(),
            rx_buf: [0; RX_BYTES],
            tx_buf: [0; TX_BYTES],
        }
    }

    /// Returns the endpoint peer state.
    pub fn peer_state(&self) -> PeerState {
        self.endpoint.peer().state()
    }

    /// Returns a shared reference to the owned async IO object.
    pub const fn io(&self) -> &T {
        &self.io
    }

    /// Returns a mutable reference to the owned async IO object.
    pub fn io_mut(&mut self) -> &mut T {
        &mut self.io
    }

    /// Consumes the adapter and returns the async IO object.
    pub fn into_inner(self) -> T {
        self.io
    }

    /// Starts a fresh frontend session.
    pub fn connect(&mut self) -> TokioResult<()> {
        self.endpoint.connect(self.now_ms())?;
        Ok(())
    }

    /// Drops the active frontend session.
    pub fn disconnect(&mut self) {
        self.endpoint.disconnect();
    }

    /// Queues an application message.
    pub fn send(&mut self, message: &[u8]) -> TokioResult<bool> {
        Ok(self.endpoint.send(message)?.is_some())
    }

    /// Reads currently available UART bytes and feeds them into MSRT.
    pub async fn receive_available(&mut self) -> TokioResult<usize> {
        let mut total = 0;
        loop {
            let now_ms = self.now_ms();
            let mut read_buf = ReadBuf::new(&mut self.rx_buf);
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            match Pin::new(&mut self.io).poll_read(&mut context, &mut read_buf) {
                Poll::Ready(Ok(())) if read_buf.filled().is_empty() => return Ok(total),
                Poll::Ready(Ok(())) => {
                    let n = read_buf.filled().len();
                    total += n;
                    let _ = self.endpoint.receive(now_ms, read_buf.filled());
                    if n < self.rx_buf.len() {
                        return Ok(total);
                    }
                }
                Poll::Ready(Err(error)) if error.kind() == ErrorKind::WouldBlock => {
                    return Ok(total);
                }
                Poll::Ready(Err(error)) if error.kind() == ErrorKind::Interrupted => continue,
                Poll::Ready(Err(error)) => return Err(error.into()),
                Poll::Pending => return Ok(total),
            }
        }
    }

    /// Polls one frontend event and writes pending UART bytes.
    pub async fn poll(&mut self) -> TokioResult<UartFrontendEvent> {
        let now_ms = self.now_ms();
        loop {
            match self.endpoint.poll(now_ms, &mut self.tx_buf)? {
                EndpointPoll::Transmit { bytes, .. } => self.io.write_all(bytes).await?,
                EndpointPoll::Message(message) => return Ok(UartFrontendEvent::Message(message)),
                EndpointPoll::SendFailed(failed) => {
                    return Ok(UartFrontendEvent::SendFailed(failed));
                }
                EndpointPoll::Idle => return Ok(UartFrontendEvent::Idle),
            }
        }
    }

    /// Runs `receive_available` followed by `poll`.
    pub async fn tick(&mut self) -> TokioResult<UartFrontendEvent> {
        if let Err(error) = self.receive_available().await {
            if recoverable_transport_error(&error) {
                return Ok(UartFrontendEvent::TransportUnavailable);
            }
            return Err(error);
        }

        match self.poll().await {
            Ok(event) => Ok(event),
            Err(error) if recoverable_transport_error(&error) => {
                Ok(UartFrontendEvent::TransportUnavailable)
            }
            Err(error) => Err(error),
        }
    }

    fn now_ms(&self) -> u64 {
        self.start
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }
}

fn recoverable_transport_error(error: &TokioError) -> bool {
    let TokioError::Io(error) = error else {
        return false;
    };

    matches!(
        error.kind(),
        ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::TimedOut
    )
}
