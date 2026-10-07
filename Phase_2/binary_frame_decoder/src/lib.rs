//! Custom Binary Frame Decoder
//!
//! A zero-copy network packet frame decoder designed with strict error handling,
//! endianness management, and zero `.unwrap()` or `.expect()` calls in production code.

pub mod error;
pub mod frame;

pub use error::FrameError;
pub use frame::{Frame, MessageType};

#[cfg(test)]
mod tests;
