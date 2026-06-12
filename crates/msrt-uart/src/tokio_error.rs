//! Error types for Tokio UART frontend.

use std::io;

/// Result alias for Tokio UART frontend operations.
pub type TokioResult<T> = core::result::Result<T, TokioError>;

/// Error returned by Tokio UART frontend.
#[derive(Debug)]
pub enum TokioError {
    /// Underlying async IO failed.
    Io(io::Error),
    /// MSRT protocol processing failed.
    Protocol(msrt::error::Error),
}

impl From<io::Error> for TokioError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<msrt::error::Error> for TokioError {
    fn from(error: msrt::error::Error) -> Self {
        Self::Protocol(error)
    }
}
