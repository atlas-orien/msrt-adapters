//! Multi-peer UDP server adapter.

use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Instant;

use msrt::endpoint::{EndpointPoll, EngineConfig, PeerState, ServerEndpoint};
use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::error::{Error, Result};
use crate::event::UdpServerEvent;

const RX_BYTES: usize = 2048;
const TX_BYTES: usize = 256;

/// Multi-peer UDP server adapter.
#[derive(Debug)]
pub struct UdpServer<const N: usize> {
    socket: UdpSocket,
    endpoint: ServerEndpoint<SocketAddr, N>,
    start: Instant,
    rx_buf: [u8; RX_BYTES],
    tx_buf: [u8; TX_BYTES],
}

impl<const N: usize> UdpServer<N> {
    /// Binds a UDP server socket using default MSRT config.
    pub async fn bind<A>(local: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        Self::bind_with_config(local, EngineConfig::default()).await
    }

    /// Binds a UDP server socket using `config`.
    pub async fn bind_with_config<A>(local: A, config: EngineConfig) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let socket = UdpSocket::bind(local).await?;
        Ok(Self::from_socket(socket, config))
    }

    /// Creates a server from an existing UDP socket.
    pub fn from_socket(socket: UdpSocket, config: EngineConfig) -> Self {
        Self {
            socket,
            endpoint: ServerEndpoint::new(config),
            start: Instant::now(),
            rx_buf: [0; RX_BYTES],
            tx_buf: [0; TX_BYTES],
        }
    }

    /// Returns the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
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

    /// Returns the peer state for `peer`.
    pub fn peer_state(&mut self, peer: SocketAddr) -> Option<PeerState> {
        self.endpoint.peer_mut(peer).map(|slot| slot.state())
    }

    /// Returns the currently accepted peers.
    pub fn peers(&self) -> impl Iterator<Item = SocketAddr> + '_ {
        self.endpoint.peers().map(|peer| *peer.peer_id())
    }

    /// Queues an application message for `peer`.
    pub fn send_to(&mut self, peer: SocketAddr, message: &[u8]) -> Result<bool> {
        match self.endpoint.send(peer, message) {
            Some(Ok(Some(_))) => Ok(true),
            Some(Ok(None)) | None => Ok(false),
            Some(Err(error)) => Err(error.into()),
        }
    }

    /// Disconnects a peer and frees its server slot.
    pub fn disconnect(&mut self, peer: SocketAddr) -> bool {
        self.endpoint.disconnect(peer)
    }

    /// Disconnects every peer idle for at least `timeout_ms`.
    pub fn disconnect_idle(&mut self, timeout_ms: u64) -> usize {
        self.endpoint.disconnect_idle(self.now_ms(), timeout_ms)
    }

    /// Receives currently available UDP datagrams and feeds them into MSRT.
    ///
    /// Unknown peers are accepted automatically. If the fixed-capacity peer
    /// table is full, `Error::Accept` is returned.
    pub fn receive_available(&mut self) -> Result<usize> {
        let mut datagrams = 0;
        loop {
            match self.socket.try_recv_from(&mut self.rx_buf) {
                Ok((n, peer)) => {
                    datagrams += 1;
                    let now_ms = self.now_ms();
                    if self.endpoint.peer_mut(peer).is_none() {
                        self.endpoint.accept(peer, now_ms)?;
                    }
                    let _ = self.endpoint.receive(peer, now_ms, &self.rx_buf[..n]);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(datagrams),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    /// Polls one adapter event and sends pending UDP datagrams.
    pub async fn poll(&mut self) -> Result<UdpServerEvent> {
        let now_ms = self.now_ms();
        let peers: Vec<SocketAddr> = self.peers().collect();

        for peer in peers {
            loop {
                let poll = self
                    .endpoint
                    .poll(peer, now_ms, &mut self.tx_buf)
                    .ok_or_else(|| Error::Io(ErrorKind::NotFound.into()))??;

                match poll {
                    EndpointPoll::Transmit { bytes, .. } => {
                        let _ = self.socket.send_to(bytes, peer).await?;
                    }
                    EndpointPoll::Message(message) => {
                        return Ok(UdpServerEvent::Message { peer, message });
                    }
                    EndpointPoll::SendFailed(failed) => {
                        return Ok(UdpServerEvent::SendFailed { peer, failed });
                    }
                    EndpointPoll::Idle => break,
                }
            }
        }

        Ok(UdpServerEvent::Idle)
    }

    /// Runs `receive_available` followed by `poll`.
    pub async fn tick(&mut self) -> Result<UdpServerEvent> {
        let _ = self.receive_available()?;
        self.poll().await
    }

    fn now_ms(&self) -> u64 {
        self.start
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }
}
