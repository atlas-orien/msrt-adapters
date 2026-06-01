#![no_std]
#![doc = "Embassy-friendly UART adapter boundary for Serial Realtime Transport."]

mod driver;
mod error;
mod io_pump;
mod send;

pub use driver::{ReceivedMessage, SendFailedEvent, SendFailedReason, UartDriver};
pub use error::{Error, ErrorKind, Result, UartErrorKind};
