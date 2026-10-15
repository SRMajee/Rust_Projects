#include "crypto_ops.h"

int32_t c_compute_checksum(const uint8_t *data, size_t len, uint32_t *out_checksum) {
    if (!data || !out_checksum) {
        return -1;
    }

    uint32_t hash = 5381;
    for (size_t i = 0; i < len; ++i) {
        hash = ((hash << 5) + hash) + data[i]; // hash * 33 + c
    }

    *out_checksum = hash;
    return 0;
}

int32_t c_apply_mask(uint8_t *data, size_t len, uint8_t mask) {
    if (!data) {
        return -1;
    }

    for (size_t i = 0; i < len; ++i) {
        data[i] ^= mask;
    }

    return 0;
}
