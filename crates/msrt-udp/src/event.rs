//! UDP adapter events.

use std::io::ErrorKind;
use std::net::SocketAddr;

use msrt::endpoint::{MessageEvent, SendFailedEvent};

/// High-level event produced by a UDP client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UdpClientEvent {
    /// A complete application message was received.
    Message(MessageEvent),
    /// Reliable send failed.
    SendFailed(SendFailedEvent),
    /// The connected UDP peer is currently unreachable.
    ///
    /// This is a recoverable transport condition, such as a restarted server
    /// causing ICMP port-unreachable feedback on connected UDP sockets.
    TransportUnavailable {
        /// Underlying socket error kind.
        kind: ErrorKind,
    },
    /// No action is currently pending.
    Idle,
}

/// High-level event produced by a UDP server.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UdpServerEvent {
    /// A complete application message was received from a peer.
    Message {
        /// Peer address that sent the message.
        peer: SocketAddr,
        /// Reassembled MSRT message.
        message: MessageEvent,
    },
    /// Reliable send failed for a peer.
    SendFailed {
        /// Peer address whose send failed.
        peer: SocketAddr,
        /// Send failure details.
        failed: SendFailedEvent,
    },
    /// No action is currently pending.
    Idle,
}
