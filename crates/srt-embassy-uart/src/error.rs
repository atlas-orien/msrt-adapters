use srt::{SendFailed, core::Error as SrtError};

/// Broad error category for `srt-embassy-uart` failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// UART read operation failed.
    UartRead,
    /// UART write operation failed.
    UartWrite,
    /// UART flush operation failed.
    UartFlush,
    /// SRT protocol operation failed.
    Protocol,
    /// Reliable send failed in SRT engine.
    SendFailed,
    /// Global driver helper is not initialized.
    NotInitialized,
    /// Global driver helper was initialized twice.
    AlreadyInitialized,
    /// Global driver helper is temporarily unavailable.
    GlobalBusy,
    /// Internal received-message queue is full.
    MessageQueueFull,
    /// Internal send-failed queue is full.
    SendFailedQueueFull,
}

/// Coarse UART error classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartErrorKind {
    /// Underlying bus or device is not connected.
    NotConnected,
    /// Requested operation is not supported.
    Unsupported,
    /// Operation timed out.
    TimedOut,
    /// Any other UART error category.
    Other,
}

impl From<embedded_io_async::ErrorKind> for UartErrorKind {
    fn from(value: embedded_io_async::ErrorKind) -> Self {
        match value {
            embedded_io_async::ErrorKind::NotConnected => Self::NotConnected,
            embedded_io_async::ErrorKind::Unsupported => Self::Unsupported,
            embedded_io_async::ErrorKind::TimedOut => Self::TimedOut,
            _ => Self::Other,
        }
    }
}

/// Shared adapter error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    uart_error_kind: Option<UartErrorKind>,
    protocol_error: Option<SrtError>,
    send_failed: Option<SendFailed>,
}

impl Error {
    /// Creates a new adapter error from kind.
    #[must_use]
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            uart_error_kind: None,
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Returns the broad error category.
    #[must_use]
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    /// Returns UART error classification if this is a UART error.
    #[must_use]
    pub const fn uart_error_kind(self) -> Option<UartErrorKind> {
        self.uart_error_kind
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

    /// Creates a UART read error.
    #[must_use]
    pub const fn uart_read(kind: UartErrorKind) -> Self {
        Self {
            kind: ErrorKind::UartRead,
            uart_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates a UART write error.
    #[must_use]
    pub const fn uart_write(kind: UartErrorKind) -> Self {
        Self {
            kind: ErrorKind::UartWrite,
            uart_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates a UART flush error.
    #[must_use]
    pub const fn uart_flush(kind: UartErrorKind) -> Self {
        Self {
            kind: ErrorKind::UartFlush,
            uart_error_kind: Some(kind),
            protocol_error: None,
            send_failed: None,
        }
    }

    /// Creates a protocol error.
    #[must_use]
    pub const fn protocol(error: SrtError) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            uart_error_kind: None,
            protocol_error: Some(error),
            send_failed: None,
        }
    }

    /// Creates a reliable-send failure error.
    #[must_use]
    pub const fn send_failed_error(failed: SendFailed) -> Self {
        Self {
            kind: ErrorKind::SendFailed,
            uart_error_kind: None,
            protocol_error: None,
            send_failed: Some(failed),
        }
    }

    /// Creates a UART read error from a concrete UART error value.
    #[must_use]
    pub fn uart_read_from<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::uart_read(UartErrorKind::from(embedded_io_async::Error::kind(&error)))
    }

    /// Creates a UART write error from a concrete UART error value.
    #[must_use]
    pub fn uart_write_from<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::uart_write(UartErrorKind::from(embedded_io_async::Error::kind(&error)))
    }

    /// Creates a UART flush error from a concrete UART error value.
    #[must_use]
    pub fn uart_flush_from<E>(error: E) -> Self
    where
        E: embedded_io_async::Error,
    {
        Self::uart_flush(UartErrorKind::from(embedded_io_async::Error::kind(&error)))
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

/// Shared result type for `srt-embassy-uart`.
pub type Result<T> = core::result::Result<T, Error>;
