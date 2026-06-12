//! UART IO traits shared by std and no-std adapters.

/// Result alias for UART IO operations.
pub type UartIoResult<T, E> = core::result::Result<T, UartIoError<E>>;

/// Result alias for UART adapter operations.
pub type UartResult<T, E> = core::result::Result<T, UartError<E>>;

/// UART adapter error wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartError<E> {
    /// UART transport failed.
    Io(UartIoError<E>),
    /// MSRT protocol processing failed.
    Protocol(msrt::error::Error),
}

/// UART transport error wrapper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UartIoError<E> {
    /// No bytes are currently available.
    WouldBlock,
    /// Operation was interrupted and may be retried.
    Interrupted,
    /// Transport-specific error.
    Other(E),
}

/// Minimal UART byte-stream trait for MCU/no-std backends.
///
/// Implement this trait for a board UART, DMA ring, RTOS queue, or test double.
pub trait UartIo {
    /// Transport-specific error type.
    type Error;

    /// Reads currently available bytes into `buf`.
    ///
    /// Return `Ok(0)` for EOF-like idle streams, or `Err(WouldBlock)` when no
    /// bytes are available right now.
    fn read(&mut self, buf: &mut [u8]) -> UartIoResult<usize, Self::Error>;

    /// Writes all bytes to the UART transport.
    fn write_all(&mut self, bytes: &[u8]) -> UartIoResult<(), Self::Error>;
}
