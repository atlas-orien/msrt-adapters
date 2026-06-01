use srt::{SendFailed, core::Error as SrtError};

/// Broad error category for SRT adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Read operation failed.
    IoRead,
    /// Write operation failed.
    IoWrite,
    /// Flush operation failed.
    IoFlush,
    /// SRT protocol operation failed.
    Protocol,
    /// Reliable send failed in SRT engine.
    SendFailed,
    /// Singleton API is not initialized.
    NotInitialized,
    /// Singleton API was initialized twice.
    AlreadyInitialized,
    /// Singleton API is temporarily unavailable.
    GlobalBusy,
    /// Internal received-message queue is full.
    MessageQueueFull,
    /// Internal send-failed queue is full.
    SendFailedQueueFull,
}

/// Coarse I/O error classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IoErrorKind {
    /// Underlying bus or device is not connected.
    NotConnected,
    /// Requested operation is not supported.
    Unsupported,
    /// Operation timed out.
    TimedOut,
    /// Any other I/O error category.
    Other,
}

/// Shared adapter error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    io_error_kind: Option<IoErrorKind>,
    protocol_error: Option<SrtError>,
    send_failed: Option<SendFailed>,
}

impl Error {
    /// Creates a new adapter error from kind.
    #[must_use]
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            io_error_kind: None,
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Returns the broad error category.
    #[must_use]
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    /// Returns I/O error classification if this is an I/O error.
    #[must_use]
    pub const fn io_error_kind(self) -> Option<IoErrorKind> {
        self.io_error_kind
    }

    /// Returns the embedded SRT protocol error if present.
    #[must_use]
    pub const fn protocol_error(self) -> Option<SrtError> {
        self.protocol_error
    }

    /// Returns reliable-send failure details if present.
    #[must_use]
    pub const fn send_failed(self) -> Option<SendFailed> {
        self.send_failed
    }

    /// Creates an I/O read error.
    #[must_use]
    pub const fn io_read(kind: IoErrorKind) -> Self {
        Self {
            kind: ErrorKind::IoRead,
            io_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates an I/O write error.
    #[must_use]
    pub const fn io_write(kind: IoErrorKind) -> Self {
        Self {
            kind: ErrorKind::IoWrite,
            io_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates an I/O flush error.
    #[must_use]
    pub const fn io_flush(kind: IoErrorKind) -> Self {
        Self {
            kind: ErrorKind::IoFlush,
            io_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates an I/O read error from an embedded-io error.
    #[must_use]
    pub fn embedded_io_read<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::io_read(IoErrorKind::from_embedded_io(error))
    }

    /// Creates an I/O write error from an embedded-io error.
    #[must_use]
    pub fn embedded_io_write<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::io_write(IoErrorKind::from_embedded_io(error))
    }

    /// Creates an I/O flush error from an embedded-io error.
    #[must_use]
    pub fn embedded_io_flush<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::io_flush(IoErrorKind::from_embedded_io(error))
    }

    /// Creates a protocol error.
    #[must_use]
    pub const fn protocol(error: SrtError) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            io_error_kind: None,
            protocol_error: Some(error),
            send_failed: None,
        }
    }

    /// Creates a reliable-send failure error.
    #[must_use]
    pub const fn send_failed_error(failed: SendFailed) -> Self {
        Self {
            kind: ErrorKind::SendFailed,
            io_error_kind: None,
            protocol_error: None,
            send_failed: Some(failed),
        }
    }
}

impl IoErrorKind {
    /// Converts an embedded-io error into an adapter I/O error kind.
    #[must_use]
    pub fn from_embedded_io<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        match embedded_io_async::Error::kind(&error) {
            embedded_io_async::ErrorKind::NotConnected => Self::NotConnected,
            embedded_io_async::ErrorKind::Unsupported => Self::Unsupported,
            embedded_io_async::ErrorKind::TimedOut => Self::TimedOut,
            _ => Self::Other,
        }
    }
}

impl From<SrtError> for Error {
    fn from(value: SrtError) -> Self {
        Self::protocol(value)
    }
}

impl From<SendFailed> for Error {
    fn from(value: SendFailed) -> Self {
        Self::send_failed_error(value)
    }
}

/// Shared result type for SRT adapters.
pub type Result<T> = core::result::Result<T, Error>;
