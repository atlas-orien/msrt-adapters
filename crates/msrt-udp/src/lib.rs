//! Tokio UDP adapters for MSRT.
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::large_enum_variant)]

mod client;
mod error;
mod event;
mod server;
#[cfg(test)]
mod tests;

pub use client::UdpClient;
pub use error::{Error, Result};
pub use event::{UdpClientEvent, UdpServerEvent};
pub use msrt::endpoint::{
    AcceptError, EngineConfig, IntegrityConfig, MessageEvent, PeerState, SendFailedEvent,
};
pub use server::UdpServer;
