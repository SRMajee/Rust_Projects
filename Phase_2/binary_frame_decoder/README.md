# Custom Binary Frame Decoder

A zero-copy network frame decoder implementing idiomatic error handling, endianness handling, and stream parsing.

## Wire Protocol Layout

```
+---------------+---------------+--------------------+------------------+------------------+
| Magic (2B)    | Version (1B)  | Message Type (1B)  | Length (2B, BE)  | Payload (N B)    | Checksum (1B)    |
| 0xAA 0x55     | 0x01          | 0x01..0x03         | 0x0000..0xFFFF   | ...              | XOR Checksum     |
+---------------+---------------+--------------------+------------------+------------------+
| <---------------------------- Header: 6 Bytes ----------------------->| <-- N Bytes -->  | <-- 1 Byte ----> |
```

## Features

- **Zero `.unwrap()` or `.expect()`**: Non-test source code returns rich domain errors (`FrameError`).
- **Zero-Copy**: Borrowed payload slices (`&'a [u8]`) using Rust lifetimes.
- **Network Byte Order**: Big-Endian integer conversions (`u16::from_be_bytes`).
- **Stream Friendly**: `Frame::decode` yields `(Frame<'a>, &'a [u8])`, allowing consecutive packets to be peeled off a continuous stream.
- **Defensive Bounds**: Rejects packets larger than `MAX_PAYLOAD_LEN` before memory exhaustion or misinterpretation can occur.
