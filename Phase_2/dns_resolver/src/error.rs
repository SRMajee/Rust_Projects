//! Custom error types for the DNS resolver.

use std::fmt;
use std::io;

/// Represents errors that can occur during DNS query construction, network transport, or packet parsing.
#[derive(Debug)]
pub enum DnsError {
    /// Underlying I/O error (e.g. socket bind, send, or timeout).
    Io(io::Error),
    /// Domain name is empty, exceeds 255 bytes, or has labels longer than 63 bytes.
    InvalidDomainName(String),
    /// Packet buffer ended prematurely while parsing fields.
    BufferTooShort { expected: usize, available: usize },
    /// DNS server returned an error response code (RCODE != 0).
    ServerFailure(u8),
    /// Packet contained an unsupported or invalid compression pointer offset.
    InvalidPointer(usize),
    /// DNS response contained 0 answers.
    NoAnswers,
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::InvalidDomainName(domain) => write!(f, "Invalid domain name: '{domain}'"),
            Self::BufferTooShort {
                expected,
                available,
            } => write!(
                f,
                "Packet buffer too short: expected at least {expected} bytes, but only {available} available"
            ),
            Self::ServerFailure(rcode) => {
                let desc = match rcode {
                    1 => "Format error (FormErr)",
                    2 => "Server failure (ServFail)",
                    3 => "Name error / Domain does not exist (NXDomain)",
                    4 => "Not implemented (NotImp)",
                    5 => "Refused (Refused)",
                    _other => "Unknown DNS server error code",
                };
                write!(f, "DNS server returned error code {rcode} ({desc})")
            }
            Self::InvalidPointer(offset) => {
                write!(f, "Invalid DNS label compression pointer to offset {offset}")
            }
            Self::NoAnswers => write!(f, "DNS response contained 0 answer records"),
        }
    }
}

impl std::error::Error for DnsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for DnsError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
