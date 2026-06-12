#ifndef MSRT_FFI_H
#define MSRT_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MSRT_STATUS_OK 0
#define MSRT_STATUS_NULL -1
#define MSRT_STATUS_INVALID_SLICE -2
#define MSRT_STATUS_PROTOCOL_ERROR -3
#define MSRT_STATUS_UNSUPPORTED -4
#define MSRT_STATUS_NO_SESSION -5
#define MSRT_STATUS_INVALID_STORAGE -6

#define MSRT_ROLE_CLIENT 1u
#define MSRT_ROLE_PASSIVE 2u

#define MSRT_INTEGRITY_CRC16 1u
#define MSRT_INTEGRITY_CRC32 2u
#define MSRT_INTEGRITY_CRC64 3u
#define MSRT_INTEGRITY_SIP_TAG_DEFAULT 4u
#define MSRT_INTEGRITY_SIP_TAG_KEY 5u
#define MSRT_INTEGRITY_AEAD_DEFAULT MSRT_INTEGRITY_SIP_TAG_DEFAULT
#define MSRT_INTEGRITY_AEAD_KEY MSRT_INTEGRITY_SIP_TAG_KEY

#define MSRT_POLL_IDLE 0u
#define MSRT_POLL_TRANSMIT 1u
#define MSRT_POLL_MESSAGE 2u
#define MSRT_POLL_SEND_FAILED 3u

#define MSRT_PEER_DISCONNECTED 0u
#define MSRT_PEER_CONNECTING 1u
#define MSRT_PEER_CONNECTED 2u

#define MSRT_RECEIVE_NONE 0u
#define MSRT_RECEIVE_PACKET 1u
#define MSRT_RECEIVE_DUPLICATE 2u
#define MSRT_RECEIVE_ACK 3u
#define MSRT_RECEIVE_PING 4u
#define MSRT_RECEIVE_PONG 5u
#define MSRT_RECEIVE_NOISE 6u
#define MSRT_RECEIVE_CORRUPTED 7u
#define MSRT_RECEIVE_INCOMPLETE 8u
#define MSRT_RECEIVE_ERROR 9u

#define MSRT_EVENT_BYTES 256u
#define MSRT_ENDPOINT_STORAGE_BYTES 32768u

typedef struct msrt_endpoint msrt_endpoint_t;

typedef union msrt_endpoint_storage {
    uint64_t align;
    uint8_t bytes[MSRT_ENDPOINT_STORAGE_BYTES];
} msrt_endpoint_storage_t;

typedef struct msrt_config {
    uint32_t role;
    uint32_t initial_message_id;
    size_t fragment_bytes;
    uint8_t max_retransmit_attempts;
    uint64_t retransmit_timeout_ms;
    uint64_t reassembly_timeout_ms;
    uint32_t integrity;
    uint8_t aead_key[16];
} msrt_config_t;

typedef struct msrt_poll_event {
    uint32_t kind;
    size_t len;
    uint8_t bytes[MSRT_EVENT_BYTES];
    uint8_t attempts;
    uint8_t packet_type;
    uint32_t message_id;
    uint32_t failure_reason;
} msrt_poll_event_t;

typedef struct msrt_receive_event {
    uint32_t kind;
    size_t value;
} msrt_receive_event_t;

uint32_t msrt_ffi_version(void);
size_t msrt_endpoint_storage_size(void);
size_t msrt_endpoint_storage_align(void);
int32_t msrt_config_default(msrt_config_t *out);

int32_t msrt_endpoint_init(
    msrt_endpoint_storage_t *storage,
    const msrt_config_t *config,
    msrt_endpoint_t **out
);
int32_t msrt_endpoint_deinit(msrt_endpoint_t *endpoint);

#ifndef MSRT_FFI_NO_HEAP
msrt_endpoint_t *msrt_endpoint_new(const msrt_config_t *config);
void msrt_endpoint_free(msrt_endpoint_t *endpoint);
#endif

int32_t msrt_endpoint_connect(msrt_endpoint_t *endpoint, uint64_t now_ms);
int32_t msrt_endpoint_disconnect(msrt_endpoint_t *endpoint);
int32_t msrt_endpoint_send(msrt_endpoint_t *endpoint, const uint8_t *bytes, size_t len);
int32_t msrt_endpoint_receive(
    msrt_endpoint_t *endpoint,
    uint64_t now_ms,
    const uint8_t *bytes,
    size_t len,
    msrt_receive_event_t *out
);
int32_t msrt_endpoint_poll(
    msrt_endpoint_t *endpoint,
    uint64_t now_ms,
    msrt_poll_event_t *out
);
uint32_t msrt_endpoint_state(const msrt_endpoint_t *endpoint);

#ifdef __cplusplus
}
#endif

#endif
