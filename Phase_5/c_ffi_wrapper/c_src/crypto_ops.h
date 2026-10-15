#ifndef CRYPTO_OPS_H
#define CRYPTO_OPS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Computes a rolling polynomial checksum over a byte buffer.
// Returns 0 on success, or -1 if data is NULL.
int32_t c_compute_checksum(const uint8_t *data, size_t len, uint32_t *out_checksum);

// Inverts bytes in a buffer in-place using an XOR mask.
// Returns 0 on success, or -1 if data is NULL.
int32_t c_apply_mask(uint8_t *data, size_t len, uint8_t mask);

#ifdef __cplusplus
}
#endif

#endif // CRYPTO_OPS_H
