//! UDP network client for resolving DNS queries.

use crate::error::DnsError;
use crate::protocol::{build_query_packet, parse_answers, DnsAnswer, DnsHeader};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

/// DNS Resolver client configuring socket and query dispatching.
pub struct DnsResolver {
    server: SocketAddr,
    timeout: Duration,
}

impl DnsResolver {
    /// Creates a new resolver pointed at a specific DNS server (e.g. `8.8.8.8:53`).
    pub fn new(server: SocketAddr) -> Self {
        Self {
            server,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Sets socket read timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Resolves an `A` record for the given domain name and returns matching IP addresses.
    pub fn resolve_a(&self, domain: &str) -> Result<Vec<DnsAnswer>, DnsError> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(self.timeout))?;

        // Generate pseudo-random transaction ID from current thread or timestamp
        let tx_id = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            & 0xFFFF) as u16;

        let query = build_query_packet(domain, tx_id)?;
        socket.send_to(&query, self.server)?;

        let mut buf = [0u8; 512];
        let (bytes_received, _) = socket.recv_from(&mut buf)?;
        let response_buf = &buf[..bytes_received];

        let header = DnsHeader::parse(response_buf)?;
        if header.id != tx_id {
            return Err(DnsError::ServerFailure(0));
        }

        parse_answers(response_buf, &header)
    }
}
