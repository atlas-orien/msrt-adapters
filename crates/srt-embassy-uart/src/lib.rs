#![no_std]
#![doc = "Embassy-friendly UART adapter boundary for Serial Realtime Transport."]

mod driver;
mod error;
mod io_pump;
mod recv;
mod send;
mod tick;

pub use driver::UartDriver;
pub use error::{Error, ErrorKind, Result, UartErrorKind};
