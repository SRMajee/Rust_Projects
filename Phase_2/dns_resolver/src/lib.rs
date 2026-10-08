pub mod error;
pub mod protocol;
pub mod resolver;

pub use error::DnsError;
pub use protocol::{DnsAnswer, DnsHeader};
pub use resolver::DnsResolver;

#[cfg(test)]
mod tests;
