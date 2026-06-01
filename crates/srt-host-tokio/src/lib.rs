#![doc = "Tokio host adapter boundary for Serial Realtime Transport."]

mod driver;
mod error;
mod io_pump;
mod send;

pub use driver::HostDriver;
pub use error::{Error, ErrorKind, Result};
