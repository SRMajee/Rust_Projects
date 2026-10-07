use crate::error::FrameError;

/// Protocol synchronization magic bytes: `0xAA`, `0x55`.
pub const MAGIC_BYTES: [u8; 2] = [0xAA, 0x55];

/// Currently supported protocol version.
pub const CURRENT_VERSION: u8 = 1;

/// Fixed header length in bytes:
/// 2B Magic + 1B Version + 1B Type + 2B PayloadLength = 6 Bytes.
pub const HEADER_LEN: usize = 6;

/// 1B Checksum length at the trailer.
pub const TRAILER_LEN: usize = 1;

/// Maximum payload length allowed per frame (1 KiB).
pub const MAX_PAYLOAD_LEN: usize = 1024;

/// Recognized message types in the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Ping = 0x01,
    Data = 0x02,
    Ack = 0x03,
}

impl TryFrom<u8> for MessageType {
    type Error = FrameError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x01 => Ok(Self::Ping),
            0x02 => Ok(Self::Data),
            0x03 => Ok(Self::Ack),
            other => Err(FrameError::UnknownMessageType(other)),
        }
    }
}

/// A zero-copy representation of a decoded packet frame.
#[derive(Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    pub version: u8,
    pub msg_type: MessageType,
    pub payload: &'a [u8],
}

impl<'a> Frame<'a> {
    /// Constructs a new `Frame` reference.
    pub fn new(version: u8, msg_type: MessageType, payload: &'a [u8]) -> Self {
        Self {
            version,
            msg_type,
            payload,
        }
    }

    /// Decodes a single frame from the beginning of `input`.
    ///
    /// Returns:
    /// - `Ok((frame, remaining_slice))` on success.
    /// - `Err(FrameError)` describing exact parsing failure.
    ///
    /// # Safety and Panics
    /// Guaranteed to never panic, `.unwrap()`, or `.expect()`.
    pub fn decode(input: &'a [u8]) -> Result<(Self, &'a [u8]), FrameError> {
        // 1. Verify input contains at least the full header
        if input.len() < HEADER_LEN {
            return Err(FrameError::IncompleteHeader {
                expected: HEADER_LEN,
                actual: input.len(),
            });
        }

        // 2. Validate Magic Synchronization Bytes
        let magic = [input[0], input[1]];
        if magic != MAGIC_BYTES {
            return Err(FrameError::InvalidMagic {
                expected: MAGIC_BYTES,
                found: magic,
            });
        }

        // 3. Validate Protocol Version
        let version = input[2];
        if version != CURRENT_VERSION {
            return Err(FrameError::UnsupportedVersion(version));
        }

        // 4. Validate and convert Message Type opcode
        let msg_type = MessageType::try_from(input[3])?;

        // 5. Read Payload Length (Big-Endian network byte order)
        let payload_len = u16::from_be_bytes([input[4], input[5]]) as usize;

        // 6. Enforce maximum payload bound
        if payload_len > MAX_PAYLOAD_LEN {
            return Err(FrameError::PayloadTooLarge {
                max: MAX_PAYLOAD_LEN,
                actual: payload_len,
            });
        }

        // 7. Verify buffer has entire payload + 1B trailer checksum
        let total_frame_len = HEADER_LEN + payload_len + TRAILER_LEN;
        if input.len() < total_frame_len {
            return Err(FrameError::IncompletePayload {
                expected: payload_len + TRAILER_LEN,
                actual: input.len() - HEADER_LEN,
            });
        }

        // 8. Extract payload slice and expected checksum
        let payload = &input[HEADER_LEN..HEADER_LEN + payload_len];
        let expected_checksum = input[HEADER_LEN + payload_len];

        // 9. Verify Checksum over (header bytes + payload)
        let calculated_checksum = compute_checksum(&input[..HEADER_LEN + payload_len]);
        if expected_checksum != calculated_checksum {
            return Err(FrameError::ChecksumMismatch {
                expected: expected_checksum,
                calculated: calculated_checksum,
            });
        }

        let remaining = &input[total_frame_len..];
        Ok((
            Frame {
                version,
                msg_type,
                payload,
            },
            remaining,
        ))
    }

    /// Encodes a `Frame` into a byte vector with header and valid checksum trailer.
    pub fn encode(&self) -> Vec<u8> {
        let payload_len = self.payload.len() as u16;
        let mut buf = Vec::with_capacity(HEADER_LEN + self.payload.len() + TRAILER_LEN);

        buf.extend_from_slice(&MAGIC_BYTES);
        buf.push(self.version);
        buf.push(self.msg_type as u8);
        buf.extend_from_slice(&payload_len.to_be_bytes());
        buf.extend_from_slice(self.payload);

        let checksum = compute_checksum(&buf);
        buf.push(checksum);

        buf
    }
}

/// Computes an 8-bit XOR checksum over data bytes.
pub fn compute_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}
