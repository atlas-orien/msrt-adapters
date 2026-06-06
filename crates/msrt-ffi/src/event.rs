//! C-compatible poll and receive events.

use msrt::endpoint::{EndpointPoll, ReceiveReport};
use msrt::error::{Error, ErrorKind};

use crate::constants::{
    MAX_EVENT_BYTES, MSRT_POLL_IDLE, MSRT_POLL_MESSAGE, MSRT_POLL_SEND_FAILED, MSRT_POLL_TRANSMIT,
    MSRT_RECEIVE_ACK, MSRT_RECEIVE_CORRUPTED, MSRT_RECEIVE_DUPLICATE, MSRT_RECEIVE_ERROR,
    MSRT_RECEIVE_INCOMPLETE, MSRT_RECEIVE_NOISE, MSRT_RECEIVE_PACKET, MSRT_RECEIVE_PING,
    MSRT_RECEIVE_PONG,
};

/// C-compatible poll event.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MsrtPollEvent {
    /// Event kind: one of `MSRT_POLL_*`.
    pub kind: u32,
    /// Number of valid bytes in `bytes`.
    pub len: usize,
    /// Event byte storage for transmit bytes or received messages.
    pub bytes: [u8; MAX_EVENT_BYTES],
    /// Send attempt count for transmit events.
    pub attempts: u8,
    /// Packet type from MSRT for message or send-failed events.
    pub packet_type: u8,
    /// Message identifier for message or send-failed events.
    pub message_id: u32,
    /// Failure reason for send-failed events.
    pub failure_reason: u32,
}

/// C-compatible receive report.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MsrtReceiveEvent {
    /// Receive kind: one of `MSRT_RECEIVE_*`.
    pub kind: u32,
    /// Packet index, skipped byte count, needed byte count, or error kind.
    pub value: usize,
}

impl MsrtPollEvent {
    /// Creates an idle C poll event.
    pub(crate) const fn idle() -> Self {
        Self {
            kind: MSRT_POLL_IDLE,
            len: 0,
            bytes: [0; MAX_EVENT_BYTES],
            attempts: 0,
            packet_type: 0,
            message_id: 0,
            failure_reason: 0,
        }
    }
}

/// Converts an MSRT receive report into a C receive event.
pub(crate) fn receive_event(report: ReceiveReport) -> MsrtReceiveEvent {
    match report {
        ReceiveReport::Packet { packet_index } => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_PACKET,
            value: usize::from(packet_index.get()),
        },
        ReceiveReport::Duplicate { packet_index } => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_DUPLICATE,
            value: usize::from(packet_index.get()),
        },
        ReceiveReport::Ack { packet_index } => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_ACK,
            value: usize::from(packet_index.get()),
        },
        ReceiveReport::Ping => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_PING,
            value: 0,
        },
        ReceiveReport::Pong => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_PONG,
            value: 0,
        },
        ReceiveReport::Noise { skipped } => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_NOISE,
            value: skipped,
        },
        ReceiveReport::Corrupted => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_CORRUPTED,
            value: 0,
        },
        ReceiveReport::Incomplete { needed } => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_INCOMPLETE,
            value: needed.unwrap_or(0),
        },
        ReceiveReport::Error(error) => MsrtReceiveEvent {
            kind: MSRT_RECEIVE_ERROR,
            value: error_kind(error),
        },
    }
}

/// Converts an MSRT endpoint poll into a C poll event.
pub(crate) fn poll_event(poll: EndpointPoll<'_>) -> MsrtPollEvent {
    let mut event = MsrtPollEvent::idle();
    match poll {
        EndpointPoll::Transmit { bytes, attempts } => {
            event.kind = MSRT_POLL_TRANSMIT;
            copy_bytes(&mut event, bytes);
            event.attempts = attempts;
        }
        EndpointPoll::Message(message) => {
            event.kind = MSRT_POLL_MESSAGE;
            copy_bytes(&mut event, message.as_bytes());
            event.packet_type = message.packet_type.code();
            event.message_id = message.message_id.get();
        }
        EndpointPoll::SendFailed(failed) => {
            event.kind = MSRT_POLL_SEND_FAILED;
            event.packet_type = failed.packet_type.code();
            event.message_id = failed.message_id.get();
            event.failure_reason = 1;
        }
        EndpointPoll::Idle => {}
    }

    event
}

fn copy_bytes(event: &mut MsrtPollEvent, bytes: &[u8]) {
    let len = bytes.len().min(event.bytes.len());
    event.bytes[..len].copy_from_slice(&bytes[..len]);
    event.len = len;
}

fn error_kind(error: Error) -> usize {
    match error.kind() {
        ErrorKind::Malformed => 1,
        ErrorKind::BufferTooSmall => 2,
        ErrorKind::Packet => 3,
        ErrorKind::Reliability => 4,
        ErrorKind::Channel => 5,
        ErrorKind::Engine => 6,
        ErrorKind::Unsupported => 7,
    }
}
