use srt::{SendFailed, core::Error as SrtError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    IoRead,
    IoWrite,
    IoFlush,
    Protocol,
    SendFailed,
    MessageQueueFull,
    SendFailedQueueFull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    protocol_error: Option<SrtError>,
    send_failed: Option<SendFailed>,
}

impl Error {
    #[must_use]
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            protocol_error: None,
            send_failed: None,
        }
    }

    #[must_use]
    pub const fn kind(self) -> ErrorKind {
        self.kind
    }

    #[must_use]
    pub const fn protocol_error(self) -> Option<SrtError> {
        self.protocol_error
    }

    #[must_use]
    pub const fn send_failed(self) -> Option<SendFailed> {
        self.send_failed
    }

    #[must_use]
    pub const fn protocol(error: SrtError) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            protocol_error: Some(error),
            send_failed: None,
        }
    }

    #[must_use]
    pub const fn send_failed_error(failed: SendFailed) -> Self {
        Self {
            kind: ErrorKind::SendFailed,
            protocol_error: None,
            send_failed: Some(failed),
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

pub type Result<T> = core::result::Result<T, Error>;
