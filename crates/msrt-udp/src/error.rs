//! Error types for UDP adapters.

use std::io;

use msrt::endpoint::AcceptError;

/// Result alias for UDP adapters.
pub type Result<T> = core::result::Result<T, Error>;

/// Error returned by UDP adapters.
#[derive(Debug)]
pub enum Error {
    /// Underlying UDP socket IO failed.
    Io(io::Error),
    /// MSRT protocol processing failed.
    Protocol(msrt::error::Error),
    /// Server endpoint could not accept a peer.
    Accept(AcceptError),
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

impl From<AcceptError> for Error {
    fn from(error: AcceptError) -> Self {
        Self::Accept(error)
    }
}
