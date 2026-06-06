#![doc = "C ABI bindings for MSRT."]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![allow(clippy::std_instead_of_core)]

#[cfg(not(feature = "std"))]
use core::panic::PanicInfo;

mod abi;
mod config;
mod constants;
mod endpoint;
mod event;
mod ptr;

pub use config::MsrtConfig;
pub use constants::*;
pub use endpoint::{MsrtEndpoint, MsrtEndpointStorage};
pub use event::{MsrtPollEvent, MsrtReceiveEvent};

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use core::mem::{align_of, size_of};
    use core::ptr;

    use super::*;
    use crate::abi::{
        msrt_endpoint_connect, msrt_endpoint_deinit, msrt_endpoint_free, msrt_endpoint_new,
        msrt_endpoint_poll, msrt_endpoint_receive, msrt_endpoint_state,
    };
    use crate::endpoint::{endpoint_from_storage, free_storage_endpoint};

    #[test]
    fn endpoint_storage_layout_is_large_enough() {
        assert!(size_of::<MsrtEndpoint>() <= ENDPOINT_STORAGE_BYTES);
        assert!(align_of::<MsrtEndpoint>() <= align_of::<MsrtEndpointStorage>());
    }

    #[test]
    fn client_passive_exchange_over_ffi() {
        unsafe {
            let (client_storage, client) = endpoint_from_storage(ptr::null());

            let mut passive_config = MsrtConfig::default();
            passive_config.role = MSRT_ROLE_PASSIVE;
            let (passive_storage, passive) = endpoint_from_storage(&passive_config);

            assert_eq!(msrt_endpoint_connect(client, 1), MSRT_STATUS_OK);

            let mut event = MsrtPollEvent {
                kind: MSRT_POLL_IDLE,
                len: 0,
                bytes: [0; MAX_EVENT_BYTES],
                attempts: 0,
                packet_type: 0,
                message_id: 0,
                failure_reason: 0,
            };
            assert_eq!(msrt_endpoint_poll(client, 1, &mut event), MSRT_STATUS_OK);
            assert_eq!(event.kind, MSRT_POLL_TRANSMIT);

            let mut receive = MsrtReceiveEvent {
                kind: MSRT_RECEIVE_NONE,
                value: 0,
            };
            assert_eq!(
                msrt_endpoint_receive(passive, 2, event.bytes.as_ptr(), event.len, &mut receive),
                MSRT_STATUS_OK
            );

            assert_eq!(msrt_endpoint_poll(passive, 2, &mut event), MSRT_STATUS_OK);
            assert_eq!(event.kind, MSRT_POLL_TRANSMIT);

            assert_eq!(
                msrt_endpoint_receive(client, 3, event.bytes.as_ptr(), event.len, &mut receive),
                MSRT_STATUS_OK
            );
            assert_eq!(msrt_endpoint_state(client), MSRT_PEER_CONNECTED);

            free_storage_endpoint(client_storage, client);
            free_storage_endpoint(passive_storage, passive);
        }
    }

    #[test]
    fn init_and_deinit_symbols_accept_storage_endpoint() {
        unsafe {
            let (storage, endpoint) = endpoint_from_storage(ptr::null());
            assert_eq!(msrt_endpoint_deinit(endpoint), MSRT_STATUS_OK);
            drop(Box::from_raw(storage));
        }
    }

    #[test]
    fn heap_endpoint_api_is_available_with_std() {
        unsafe {
            let endpoint = msrt_endpoint_new(ptr::null());
            assert!(!endpoint.is_null());
            msrt_endpoint_free(endpoint);
        }
    }
}
