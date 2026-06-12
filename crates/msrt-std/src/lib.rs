//! std byte-stream adapters for MSRT.

mod backend;
mod error;
mod event;
mod frontend;
mod io;
#[cfg(test)]
mod tests;

pub use backend::StdBackend;
pub use error::{Error, Result};
pub use event::AdapterEvent;
pub use frontend::StdFrontend;
pub use msrt::endpoint::{EngineConfig, IntegrityConfig, MessageEvent, PeerState, SendFailedEvent};
