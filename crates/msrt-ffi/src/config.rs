//! C-compatible endpoint configuration.

use msrt::endpoint::{EngineConfig, IntegrityConfig, MessageId};

use crate::constants::{
    MSRT_INTEGRITY_AEAD_DEFAULT, MSRT_INTEGRITY_AEAD_KEY, MSRT_INTEGRITY_CRC16,
    MSRT_INTEGRITY_CRC32, MSRT_INTEGRITY_CRC64, MSRT_ROLE_CLIENT,
};

/// C-compatible endpoint configuration.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MsrtConfig {
    /// Endpoint role: `MSRT_ROLE_CLIENT` or `MSRT_ROLE_PASSIVE`.
    pub role: u32,
    /// First message identifier used by the endpoint.
    pub initial_message_id: u32,
    /// Maximum message fragment bytes written into one packet. Zero uses the MSRT default.
    pub fragment_bytes: usize,
    /// Maximum retransmission attempts. Zero uses the MSRT default.
    pub max_retransmit_attempts: u8,
    /// Retransmission timeout in milliseconds. Zero uses the MSRT default.
    pub retransmit_timeout_ms: u64,
    /// Reassembly timeout in milliseconds. Zero uses the MSRT default.
    pub reassembly_timeout_ms: u64,
    /// Integrity backend.
    pub integrity: u32,
    /// Key used when `integrity` is `MSRT_INTEGRITY_AEAD_KEY`.
    pub aead_key: [u8; 16],
}

impl Default for MsrtConfig {
    fn default() -> Self {
        let config = EngineConfig::default();
        Self {
            role: MSRT_ROLE_CLIENT,
            initial_message_id: 0,
            fragment_bytes: config.fragment_bytes,
            max_retransmit_attempts: config.max_retransmit_attempts,
            retransmit_timeout_ms: config.retransmit_timeout_ms,
            reassembly_timeout_ms: config.reassembly_timeout_ms,
            integrity: MSRT_INTEGRITY_CRC16,
            aead_key: [0; 16],
        }
    }
}

impl MsrtConfig {
    /// Converts C configuration into MSRT engine configuration.
    pub(crate) fn engine_config(self) -> Result<EngineConfig, ()> {
        let mut config = EngineConfig::default();
        config.initial_message_id = MessageId::new(self.initial_message_id);
        if self.fragment_bytes != 0 {
            config.fragment_bytes = self.fragment_bytes;
        }
        if self.max_retransmit_attempts != 0 {
            config.max_retransmit_attempts = self.max_retransmit_attempts;
        }
        if self.retransmit_timeout_ms != 0 {
            config.retransmit_timeout_ms = self.retransmit_timeout_ms;
        }
        if self.reassembly_timeout_ms != 0 {
            config.reassembly_timeout_ms = self.reassembly_timeout_ms;
        }
        config.integrity = match self.integrity {
            0 | MSRT_INTEGRITY_CRC16 => IntegrityConfig::crc16(),
            MSRT_INTEGRITY_CRC32 => IntegrityConfig::crc32(),
            MSRT_INTEGRITY_CRC64 => IntegrityConfig::crc64(),
            MSRT_INTEGRITY_AEAD_DEFAULT => IntegrityConfig::aead(),
            MSRT_INTEGRITY_AEAD_KEY => IntegrityConfig::aead_with_key(self.aead_key),
            _ => return Err(()),
        };

        Ok(config)
    }
}
