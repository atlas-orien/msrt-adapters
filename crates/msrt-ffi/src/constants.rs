//! C ABI constants.

/// Maximum bytes carried by one C poll event.
pub const MAX_EVENT_BYTES: usize = 256;
/// Conservative caller-owned endpoint storage size.
pub const ENDPOINT_STORAGE_BYTES: usize = 32 * 1024;

/// Operation completed successfully.
pub const MSRT_STATUS_OK: i32 = 0;
/// A required pointer argument was null.
pub const MSRT_STATUS_NULL: i32 = -1;
/// A slice pointer was null while its length was non-zero.
pub const MSRT_STATUS_INVALID_SLICE: i32 = -2;
/// MSRT reported a protocol error.
pub const MSRT_STATUS_PROTOCOL_ERROR: i32 = -3;
/// The requested option value is not supported.
pub const MSRT_STATUS_UNSUPPORTED: i32 = -4;
/// The endpoint is not currently connected or accepted.
pub const MSRT_STATUS_NO_SESSION: i32 = -5;
/// Caller-provided storage is too small or insufficiently aligned.
pub const MSRT_STATUS_INVALID_STORAGE: i32 = -6;

/// Active client endpoint role.
pub const MSRT_ROLE_CLIENT: u32 = 1;
/// Passive single-peer endpoint role.
pub const MSRT_ROLE_PASSIVE: u32 = 2;

/// Default CRC-16/XMODEM integrity.
pub const MSRT_INTEGRITY_CRC16: u32 = 1;
/// CRC-32/ISO-HDLC integrity.
pub const MSRT_INTEGRITY_CRC32: u32 = 2;
/// CRC-64/ECMA-182 integrity.
pub const MSRT_INTEGRITY_CRC64: u32 = 3;
/// Lightweight keyed integrity using the MSRT default key.
pub const MSRT_INTEGRITY_AEAD_DEFAULT: u32 = 4;
/// Lightweight keyed integrity using `aead_key`.
pub const MSRT_INTEGRITY_AEAD_KEY: u32 = 5;

/// No endpoint action is pending.
pub const MSRT_POLL_IDLE: u32 = 0;
/// Event bytes should be transmitted by the caller.
pub const MSRT_POLL_TRANSMIT: u32 = 1;
/// A complete application message is available.
pub const MSRT_POLL_MESSAGE: u32 = 2;
/// A reliable message failed to send.
pub const MSRT_POLL_SEND_FAILED: u32 = 3;

/// Endpoint is disconnected.
pub const MSRT_PEER_DISCONNECTED: u32 = 0;
/// Endpoint has a local session but has not yet observed valid peer traffic.
pub const MSRT_PEER_CONNECTING: u32 = 1;
/// Endpoint has observed valid peer traffic.
pub const MSRT_PEER_CONNECTED: u32 = 2;

/// No receive detail is available.
pub const MSRT_RECEIVE_NONE: u32 = 0;
/// A packet was accepted.
pub const MSRT_RECEIVE_PACKET: u32 = 1;
/// A duplicate packet was accepted and acknowledged.
pub const MSRT_RECEIVE_DUPLICATE: u32 = 2;
/// An ACK packet was accepted.
pub const MSRT_RECEIVE_ACK: u32 = 3;
/// A PING packet was accepted.
pub const MSRT_RECEIVE_PING: u32 = 4;
/// A PONG packet was accepted.
pub const MSRT_RECEIVE_PONG: u32 = 5;
/// Noise was skipped.
pub const MSRT_RECEIVE_NOISE: u32 = 6;
/// A corrupted packet envelope was rejected.
pub const MSRT_RECEIVE_CORRUPTED: u32 = 7;
/// More bytes are needed.
pub const MSRT_RECEIVE_INCOMPLETE: u32 = 8;
/// The packet was valid but could not be applied.
pub const MSRT_RECEIVE_ERROR: u32 = 9;
