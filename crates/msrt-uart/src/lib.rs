#![doc = "UART adapters for MSRT."]
#![cfg_attr(not(feature = "std"), no_std)]

mod backend;
mod event;
mod traits;

#[cfg(feature = "tokio")]
mod frontend;
#[cfg(all(test, feature = "tokio"))]
mod tests;
#[cfg(feature = "tokio")]
mod tokio_error;

pub use backend::UartBackend;
pub use event::{UartBackendEvent, UartFrontendEvent};
pub use msrt::endpoint::{EngineConfig, IntegrityConfig, MessageEvent, PeerState, SendFailedEvent};
pub use traits::{UartError, UartIo, UartIoError, UartIoResult, UartResult};

#[cfg(feature = "tokio")]
pub use frontend::TokioUartFrontend;
#[cfg(feature = "tokio")]
pub use tokio_error::{TokioError, TokioResult};
