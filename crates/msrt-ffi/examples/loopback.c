#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "msrt_ffi.h"

static int feed(msrt_endpoint_t *dst, uint64_t now_ms, const msrt_poll_event_t *event) {
    if (event->kind != MSRT_POLL_TRANSMIT) {
        return 0;
    }

    msrt_receive_event_t receive_event = {0};
    return msrt_endpoint_receive(dst, now_ms, event->bytes, event->len, &receive_event);
}

static int poll_until_idle(msrt_endpoint_t *src, msrt_endpoint_t *dst, uint64_t now_ms) {
    for (;;) {
        msrt_poll_event_t event = {0};
        int status = msrt_endpoint_poll(src, now_ms, &event);
        if (status != MSRT_STATUS_OK) {
            return status;
        }

        if (event.kind == MSRT_POLL_IDLE) {
            return MSRT_STATUS_OK;
        }

        if (event.kind == MSRT_POLL_TRANSMIT) {
            status = feed(dst, now_ms, &event);
            if (status != MSRT_STATUS_OK) {
                return status;
            }
        } else if (event.kind == MSRT_POLL_MESSAGE && event.len > 1) {
            printf("message: %.*s\n", (int)event.len, event.bytes);
        } else if (event.kind == MSRT_POLL_SEND_FAILED) {
            return MSRT_STATUS_PROTOCOL_ERROR;
        }
    }
}

int main(void) {
    msrt_endpoint_storage_t client_storage;
    msrt_endpoint_t *client = NULL;
    if (msrt_endpoint_init(&client_storage, NULL, &client) != MSRT_STATUS_OK) {
        return 1;
    }

    msrt_config_t passive_config;
    msrt_config_default(&passive_config);
    passive_config.role = MSRT_ROLE_PASSIVE;
    msrt_endpoint_storage_t passive_storage;
    msrt_endpoint_t *passive = NULL;
    if (msrt_endpoint_init(&passive_storage, &passive_config, &passive) != MSRT_STATUS_OK) {
        return 1;
    }

    uint64_t now_ms = 1;
    if (msrt_endpoint_connect(client, now_ms) != MSRT_STATUS_OK) {
        return 1;
    }

    if (poll_until_idle(client, passive, ++now_ms) != MSRT_STATUS_OK) {
        return 1;
    }
    if (poll_until_idle(passive, client, ++now_ms) != MSRT_STATUS_OK) {
        return 1;
    }

    const char payload[] = "hello from C";
    if (msrt_endpoint_send(client, (const uint8_t *)payload, strlen(payload)) != MSRT_STATUS_OK) {
        return 1;
    }

    if (poll_until_idle(client, passive, ++now_ms) != MSRT_STATUS_OK) {
        return 1;
    }
    if (poll_until_idle(passive, client, ++now_ms) != MSRT_STATUS_OK) {
        return 1;
    }

    msrt_endpoint_deinit(client);
    msrt_endpoint_deinit(passive);
    return 0;
}
