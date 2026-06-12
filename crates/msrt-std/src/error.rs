//! Error types for std adapters.

use std::io;

/// Result alias for std adapters.
pub type Result<T> = core::result::Result<T, Error>;

/// Error returned by std adapters.
#[derive(Debug)]
pub enum Error {
    /// Underlying IO failed.
    Io(io::Error),
    /// MSRT protocol processing failed.
    Protocol(msrt::error::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<msrt::error::Error> for Error {
    fn from(error: msrt::error::Error) -> Self {
        Self::Protocol(error)
    }
}
