use std::error::Error;
use std::fmt;

/// Errors that may occur when decoding a binary frame.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FrameError {
    /// Input buffer is too small to contain the full header.
    IncompleteHeader { expected: usize, actual: usize },
    /// Magic synchronization bytes do not match expected `[0xAA, 0x55]`.
    InvalidMagic { expected: [u8; 2], found: [u8; 2] },
    /// Protocol version is not supported.
    UnsupportedVersion(u8),
    /// Message type opcode is not recognized.
    UnknownMessageType(u8),
    /// Frame specifies a payload larger than the remaining buffer slice.
    IncompletePayload { expected: usize, actual: usize },
    /// Payload exceeds maximum allowable limit (buffer overflow defense).
    PayloadTooLarge { max: usize, actual: usize },
    /// Checksum verification failed.
    ChecksumMismatch { expected: u8, calculated: u8 },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompleteHeader { expected, actual } => {
                write!(
                    f,
                    "Incomplete header: expected at least {expected} bytes, but got {actual}"
                )
            }
            Self::InvalidMagic { expected, found } => {
                write!(
                    f,
                    "Invalid magic sync bytes: expected [0x{:02X}, 0x{:02X}], found [0x{:02X}, 0x{:02X}]",
                    expected[0], expected[1], found[0], found[1]
                )
            }
            Self::UnsupportedVersion(v) => {
                write!(f, "Unsupported protocol version: {v}")
            }
            Self::UnknownMessageType(t) => {
                write!(f, "Unknown message type opcode: 0x{t:02X}")
            }
            Self::IncompletePayload { expected, actual } => {
                write!(
                    f,
                    "Incomplete payload: expected {expected} bytes, but only {actual} available"
                )
            }
            Self::PayloadTooLarge { max, actual } => {
                write!(
                    f,
                    "Payload length exceeds maximum allowed: {actual} bytes (max: {max})"
                )
            }
            Self::ChecksumMismatch { expected, calculated } => {
                write!(
                    f,
                    "Checksum mismatch: expected 0x{expected:02X}, calculated 0x{calculated:02X}"
                )
            }
        }
    }
}

impl Error for FrameError {}
