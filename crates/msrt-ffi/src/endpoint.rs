//! Endpoint storage and lifecycle internals.

use core::mem::{align_of, size_of};
use core::ptr;

#[cfg(feature = "std")]
use std::boxed::Box;

use msrt::endpoint::{ClientEndpoint, EngineConfig, PassiveEndpoint};

use crate::config::MsrtConfig;
use crate::constants::{
    ENDPOINT_STORAGE_BYTES, MAX_EVENT_BYTES, MSRT_ROLE_CLIENT, MSRT_ROLE_PASSIVE,
    MSRT_STATUS_INVALID_STORAGE, MSRT_STATUS_NULL, MSRT_STATUS_OK, MSRT_STATUS_UNSUPPORTED,
};

/// Caller-owned endpoint storage for heap-free C integrations.
///
/// C users should treat this as opaque storage and only pass it to
/// `msrt_endpoint_init` and `msrt_endpoint_deinit`.
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct MsrtEndpointStorage {
    pub(crate) bytes: [u8; ENDPOINT_STORAGE_BYTES],
}

/// Opaque MSRT endpoint handle.
pub struct MsrtEndpoint {
    pub(crate) inner: EndpointInner,
    pub(crate) tx_buf: [u8; MAX_EVENT_BYTES],
}

pub(crate) enum EndpointInner {
    Client(ClientEndpoint),
    Passive(PassiveEndpoint),
}

impl EndpointInner {
    pub(crate) fn new(role: u32, config: EngineConfig) -> Result<Self, ()> {
        match role {
            MSRT_ROLE_CLIENT => Ok(Self::Client(ClientEndpoint::new(config))),
            MSRT_ROLE_PASSIVE => Ok(Self::Passive(PassiveEndpoint::new(config))),
            _ => Err(()),
        }
    }
}

/// Returns the endpoint bytes required by this build.
pub const fn endpoint_storage_size() -> usize {
    size_of::<MsrtEndpoint>()
}

/// Returns the endpoint alignment required by this build.
pub const fn endpoint_storage_align() -> usize {
    align_of::<MsrtEndpoint>()
}

/// Initializes an endpoint in caller-owned storage.
pub(crate) unsafe fn init_endpoint(
    storage: *mut MsrtEndpointStorage,
    config: *const MsrtConfig,
    out: *mut *mut MsrtEndpoint,
) -> i32 {
    let Some(storage) = (unsafe { storage.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    *out = ptr::null_mut();
    if endpoint_storage_size() > ENDPOINT_STORAGE_BYTES
        || endpoint_storage_align() > align_of::<MsrtEndpointStorage>()
    {
        return MSRT_STATUS_INVALID_STORAGE;
    }

    let config = if config.is_null() {
        MsrtConfig::default()
    } else {
        unsafe { *config }
    };

    let Ok(engine_config) = config.engine_config() else {
        return MSRT_STATUS_UNSUPPORTED;
    };
    let Ok(inner) = EndpointInner::new(config.role, engine_config) else {
        return MSRT_STATUS_UNSUPPORTED;
    };

    let endpoint = ptr::from_mut(storage).cast::<MsrtEndpoint>();
    unsafe {
        endpoint.write(MsrtEndpoint {
            inner,
            tx_buf: [0; MAX_EVENT_BYTES],
        });
    }
    *out = endpoint;
    MSRT_STATUS_OK
}

/// Drops an endpoint initialized in caller-owned storage.
pub(crate) unsafe fn deinit_endpoint(endpoint: *mut MsrtEndpoint) -> i32 {
    if endpoint.is_null() {
        return MSRT_STATUS_NULL;
    }

    unsafe {
        ptr::drop_in_place(endpoint);
    }
    MSRT_STATUS_OK
}

/// Creates a heap-allocated endpoint handle.
#[cfg(feature = "std")]
pub(crate) unsafe fn new_endpoint(config: *const MsrtConfig) -> *mut MsrtEndpoint {
    let mut storage = MsrtEndpointStorage {
        bytes: [0; ENDPOINT_STORAGE_BYTES],
    };
    let mut endpoint = ptr::null_mut();
    let status = unsafe { init_endpoint(&mut storage, config, &mut endpoint) };
    if status != MSRT_STATUS_OK {
        return ptr::null_mut();
    }

    Box::into_raw(Box::new(unsafe { ptr::read(endpoint) }))
}

/// Frees a heap-allocated endpoint handle.
#[cfg(feature = "std")]
pub(crate) unsafe fn free_endpoint(endpoint: *mut MsrtEndpoint) {
    if endpoint.is_null() {
        return;
    }

    drop(unsafe { Box::from_raw(endpoint) });
}

#[cfg(test)]
pub(crate) fn endpoint_from_storage(
    config: *const MsrtConfig,
) -> (*mut MsrtEndpointStorage, *mut MsrtEndpoint) {
    let storage = Box::into_raw(Box::new(MsrtEndpointStorage {
        bytes: [0; ENDPOINT_STORAGE_BYTES],
    }));
    let mut endpoint = ptr::null_mut();
    assert_eq!(
        unsafe { init_endpoint(storage, config, &mut endpoint) },
        MSRT_STATUS_OK
    );
    (storage, endpoint)
}

#[cfg(test)]
pub(crate) unsafe fn free_storage_endpoint(
    storage: *mut MsrtEndpointStorage,
    endpoint: *mut MsrtEndpoint,
) {
    assert_eq!(unsafe { deinit_endpoint(endpoint) }, MSRT_STATUS_OK);
    drop(unsafe { Box::from_raw(storage) });
}
