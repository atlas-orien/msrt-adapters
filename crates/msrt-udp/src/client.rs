//! Connected UDP client adapter.

use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Instant;

use msrt::endpoint::{ClientEndpoint, EndpointPoll, EngineConfig, PeerState};

use crate::error::Result;
use crate::event::UdpClientEvent;

const RX_BYTES: usize = 2048;
const TX_BYTES: usize = 256;

/// Connected UDP client adapter.
#[derive(Debug)]
pub struct UdpClient {
    socket: UdpSocket,
    endpoint: ClientEndpoint,
    start: Instant,
    rx_buf: [u8; RX_BYTES],
    tx_buf: [u8; TX_BYTES],
}

impl UdpClient {
    /// Binds a local UDP socket and connects it to `remote`.
    pub fn bind<A, R>(local: A, remote: R) -> Result<Self>
    where
        A: ToSocketAddrs,
        R: ToSocketAddrs,
    {
        Self::bind_with_config(local, remote, EngineConfig::default())
    }

    /// Binds a local UDP socket, connects it to `remote`, and uses `config`.
    pub fn bind_with_config<A, R>(local: A, remote: R, config: EngineConfig) -> Result<Self>
    where
        A: ToSocketAddrs,
        R: ToSocketAddrs,
    {
        let socket = UdpSocket::bind(local)?;
        socket.connect(remote)?;
        socket.set_nonblocking(true)?;
        Ok(Self::from_socket(socket, config))
    }

    /// Creates a client from an already connected UDP socket.
    ///
    /// The socket is switched to nonblocking mode.
    pub fn from_socket(socket: UdpSocket, config: EngineConfig) -> Self {
        let _ = socket.set_nonblocking(true);
        Self {
            socket,
            endpoint: ClientEndpoint::new(config),
            start: Instant::now(),
            rx_buf: [0; RX_BYTES],
            tx_buf: [0; TX_BYTES],
        }
    }

    /// Returns the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }

    /// Returns the remote socket address.
    pub fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.peer_addr()?)
    }

    /// Returns the endpoint peer state.
    pub fn peer_state(&self) -> PeerState {
        self.endpoint.peer().state()
    }

    /// Returns a shared reference to the UDP socket.
    pub const fn socket(&self) -> &UdpSocket {
        &self.socket
    }

    /// Returns a mutable reference to the UDP socket.
    pub fn socket_mut(&mut self) -> &mut UdpSocket {
        &mut self.socket
    }

    /// Consumes the adapter and returns the UDP socket.
    pub fn into_socket(self) -> UdpSocket {
        self.socket
    }

    /// Starts a fresh client session.
    pub fn connect(&mut self) -> Result<()> {
        self.endpoint.connect(self.now_ms())?;
        Ok(())
    }

    /// Drops the active client session.
    pub fn disconnect(&mut self) {
        self.endpoint.disconnect();
    }

    /// Queues an application message.
    pub fn send(&mut self, message: &[u8]) -> Result<bool> {
        Ok(self.endpoint.send(message)?.is_some())
    }

    /// Receives currently available UDP datagrams and feeds them into MSRT.
    pub fn receive_available(&mut self) -> Result<usize> {
        let mut datagrams = 0;
        loop {
            match self.socket.recv(&mut self.rx_buf) {
                Ok(n) => {
                    datagrams += 1;
                    let now_ms = self.now_ms();
                    let _ = self.endpoint.receive(now_ms, &self.rx_buf[..n]);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(datagrams),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    /// Polls one adapter event and sends pending UDP datagrams.
    pub fn poll(&mut self) -> Result<UdpClientEvent> {
        let now_ms = self.now_ms();
        loop {
            match self.endpoint.poll(now_ms, &mut self.tx_buf)? {
                EndpointPoll::Transmit { bytes, .. } => {
                    let _ = self.socket.send(bytes)?;
                }
                EndpointPoll::Message(message) => return Ok(UdpClientEvent::Message(message)),
                EndpointPoll::SendFailed(failed) => return Ok(UdpClientEvent::SendFailed(failed)),
                EndpointPoll::Idle => return Ok(UdpClientEvent::Idle),
            }
        }
    }

    /// Runs `receive_available` followed by `poll`.
    pub fn tick(&mut self) -> Result<UdpClientEvent> {
        if let Err(error) = self.receive_available() {
            if let Some(kind) = recoverable_transport_error(&error) {
                return Ok(UdpClientEvent::TransportUnavailable { kind });
            }
            return Err(error);
        }

        match self.poll() {
            Ok(event) => Ok(event),
            Err(error) => {
                if let Some(kind) = recoverable_transport_error(&error) {
                    return Ok(UdpClientEvent::TransportUnavailable { kind });
                }
                Err(error)
            }
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

fn recoverable_transport_error(error: &crate::Error) -> Option<ErrorKind> {
    let crate::Error::Io(error) = error else {
        return None;
    };

    match error.kind() {
        ErrorKind::ConnectionRefused
        | ErrorKind::ConnectionReset
        | ErrorKind::ConnectionAborted
        | ErrorKind::NotConnected
        | ErrorKind::TimedOut => Some(error.kind()),
        _ => None,
    }
}
