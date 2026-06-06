//! Exported C ABI functions.

use msrt::endpoint::{PeerState, ReceiveReport};

use crate::config::MsrtConfig;
use crate::constants::{
    MSRT_PEER_CONNECTED, MSRT_PEER_CONNECTING, MSRT_PEER_DISCONNECTED, MSRT_STATUS_INVALID_SLICE,
    MSRT_STATUS_NO_SESSION, MSRT_STATUS_NULL, MSRT_STATUS_OK, MSRT_STATUS_PROTOCOL_ERROR,
    MSRT_STATUS_UNSUPPORTED,
};
use crate::endpoint::{
    EndpointInner, MsrtEndpoint, MsrtEndpointStorage, deinit_endpoint, endpoint_storage_align,
    endpoint_storage_size, init_endpoint,
};
use crate::event::{MsrtPollEvent, MsrtReceiveEvent, poll_event, receive_event};
use crate::ptr::bytes_from_raw;

#[cfg(feature = "std")]
use crate::endpoint::{free_endpoint, new_endpoint};

/// Returns the version number of the FFI crate encoded as major.minor.patch bytes.
#[unsafe(no_mangle)]
pub extern "C" fn msrt_ffi_version() -> u32 {
    (env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap_or(0) << 16)
        | (env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap_or(0) << 8)
        | env!("CARGO_PKG_VERSION_PATCH").parse::<u32>().unwrap_or(0)
}

/// Returns the storage bytes required by this build.
#[unsafe(no_mangle)]
pub extern "C" fn msrt_endpoint_storage_size() -> usize {
    endpoint_storage_size()
}

/// Returns the storage alignment required by this build.
#[unsafe(no_mangle)]
pub extern "C" fn msrt_endpoint_storage_align() -> usize {
    endpoint_storage_align()
}

/// Writes the default configuration into `out`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_config_default(out: *mut MsrtConfig) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    *out = MsrtConfig::default();
    MSRT_STATUS_OK
}

/// Initializes an endpoint in caller-owned storage and writes the endpoint handle to `out`.
///
/// This is the preferred API for MCU and no-heap C integrations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_init(
    storage: *mut MsrtEndpointStorage,
    config: *const MsrtConfig,
    out: *mut *mut MsrtEndpoint,
) -> i32 {
    unsafe { init_endpoint(storage, config, out) }
}

/// Drops an endpoint initialized by `msrt_endpoint_init`.
///
/// This releases Rust-owned protocol state inside the caller-provided storage.
/// The storage object itself remains owned by the C caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_deinit(endpoint: *mut MsrtEndpoint) -> i32 {
    unsafe { deinit_endpoint(endpoint) }
}

/// Creates a heap-allocated endpoint handle.
///
/// This convenience API is only available when the crate is built with the
/// default `std` feature. MCU integrations should use `msrt_endpoint_init`.
#[cfg(feature = "std")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_new(config: *const MsrtConfig) -> *mut MsrtEndpoint {
    unsafe { new_endpoint(config) }
}

/// Frees a heap-allocated endpoint handle. Null is accepted.
///
/// This convenience API is only available when the crate is built with the
/// default `std` feature.
#[cfg(feature = "std")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_free(endpoint: *mut MsrtEndpoint) {
    unsafe { free_endpoint(endpoint) };
}

/// Starts a fresh active client session.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_connect(endpoint: *mut MsrtEndpoint, now_ms: u64) -> i32 {
    let Some(endpoint) = (unsafe { endpoint.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    match &mut endpoint.inner {
        EndpointInner::Client(client) => status_from_result(client.connect(now_ms).map(|_| ())),
        EndpointInner::Passive(_) => MSRT_STATUS_UNSUPPORTED,
    }
}

/// Drops the active session if one exists.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_disconnect(endpoint: *mut MsrtEndpoint) -> i32 {
    let Some(endpoint) = (unsafe { endpoint.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    match &mut endpoint.inner {
        EndpointInner::Client(client) => client.disconnect(),
        EndpointInner::Passive(passive) => passive.disconnect(),
    }

    MSRT_STATUS_OK
}

/// Queues an application message for reliable transmission.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_send(
    endpoint: *mut MsrtEndpoint,
    bytes: *const u8,
    len: usize,
) -> i32 {
    let Some(endpoint) = (unsafe { endpoint.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };
    let Some(bytes) = bytes_from_raw(bytes, len) else {
        return MSRT_STATUS_INVALID_SLICE;
    };

    let result = match &mut endpoint.inner {
        EndpointInner::Client(client) => client.send(bytes),
        EndpointInner::Passive(passive) => passive.send(bytes),
    };

    match result {
        Ok(Some(_)) => MSRT_STATUS_OK,
        Ok(None) => MSRT_STATUS_NO_SESSION,
        Err(_) => MSRT_STATUS_PROTOCOL_ERROR,
    }
}

/// Feeds received bytes into the endpoint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_receive(
    endpoint: *mut MsrtEndpoint,
    now_ms: u64,
    bytes: *const u8,
    len: usize,
    out: *mut MsrtReceiveEvent,
) -> i32 {
    let Some(endpoint) = (unsafe { endpoint.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };
    let Some(bytes) = bytes_from_raw(bytes, len) else {
        return MSRT_STATUS_INVALID_SLICE;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    let report = match &mut endpoint.inner {
        EndpointInner::Client(client) => client.receive(now_ms, bytes),
        EndpointInner::Passive(passive) => passive.receive(now_ms, bytes),
    };

    *out = receive_event(report);
    if matches!(report, ReceiveReport::Error(_)) {
        MSRT_STATUS_PROTOCOL_ERROR
    } else {
        MSRT_STATUS_OK
    }
}

/// Polls one endpoint action.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_poll(
    endpoint: *mut MsrtEndpoint,
    now_ms: u64,
    out: *mut MsrtPollEvent,
) -> i32 {
    let Some(endpoint) = (unsafe { endpoint.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MSRT_STATUS_NULL;
    };

    *out = MsrtPollEvent::idle();
    let result = match &mut endpoint.inner {
        EndpointInner::Client(client) => client.poll(now_ms, &mut endpoint.tx_buf),
        EndpointInner::Passive(passive) => passive.poll(now_ms, &mut endpoint.tx_buf),
    };

    match result {
        Ok(poll) => {
            *out = poll_event(poll);
            MSRT_STATUS_OK
        }
        Err(_) => MSRT_STATUS_PROTOCOL_ERROR,
    }
}

/// Returns the endpoint peer state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn msrt_endpoint_state(endpoint: *const MsrtEndpoint) -> u32 {
    let Some(endpoint) = (unsafe { endpoint.as_ref() }) else {
        return MSRT_PEER_DISCONNECTED;
    };

    let state = match &endpoint.inner {
        EndpointInner::Client(client) => client.peer().state(),
        EndpointInner::Passive(passive) => passive.peer().state(),
    };

    match state {
        PeerState::Disconnected => MSRT_PEER_DISCONNECTED,
        PeerState::Connecting => MSRT_PEER_CONNECTING,
        PeerState::Connected => MSRT_PEER_CONNECTED,
    }
}

fn status_from_result(result: msrt::error::Result<()>) -> i32 {
    match result {
        Ok(()) => MSRT_STATUS_OK,
        Err(_) => MSRT_STATUS_PROTOCOL_ERROR,
    }
}
