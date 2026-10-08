//! Wire-format protocol parsing and building according to RFC 1035.

use crate::error::DnsError;
use std::net::Ipv4Addr;

pub const DNS_PORT: u16 = 53;
pub const HEADER_LEN: usize = 12;

/// RFC 1035 12-byte DNS Header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl DnsHeader {
    /// Builds a standard recursive query header with a specified transaction ID.
    pub fn new_query(id: u16) -> Self {
        Self {
            id,
            // RD bit (Recursion Desired) is bit 8 (0x0100)
            flags: 0x0100,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    /// Serializes the header into 12 big-endian bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut buf = [0u8; HEADER_LEN];
        buf[0..2].copy_from_slice(&self.id.to_be_bytes());
        buf[2..4].copy_from_slice(&self.flags.to_be_bytes());
        buf[4..6].copy_from_slice(&self.qdcount.to_be_bytes());
        buf[6..8].copy_from_slice(&self.ancount.to_be_bytes());
        buf[8..10].copy_from_slice(&self.nscount.to_be_bytes());
        buf[10..12].copy_from_slice(&self.arcount.to_be_bytes());
        buf
    }

    /// Parses a 12-byte header from the beginning of a buffer.
    pub fn parse(buf: &[u8]) -> Result<Self, DnsError> {
        if buf.len() < HEADER_LEN {
            return Err(DnsError::BufferTooShort {
                expected: HEADER_LEN,
                available: buf.len(),
            });
        }

        let id = u16::from_be_bytes([buf[0], buf[1]]);
        let flags = u16::from_be_bytes([buf[2], buf[3]]);
        let qdcount = u16::from_be_bytes([buf[4], buf[5]]);
        let ancount = u16::from_be_bytes([buf[6], buf[7]]);
        let nscount = u16::from_be_bytes([buf[8], buf[9]]);
        let arcount = u16::from_be_bytes([buf[10], buf[11]]);

        // Validate RCODE in response flags (lower 4 bits)
        let rcode = (flags & 0x000F) as u8;
        if rcode != 0 {
            return Err(DnsError::ServerFailure(rcode));
        }

        Ok(Self {
            id,
            flags,
            qdcount,
            ancount,
            nscount,
            arcount,
        })
    }
}

/// An IPv4 Answer record decoded from the DNS response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsAnswer {
    pub ip: Ipv4Addr,
    pub ttl: u32,
}

/// Encodes a domain name like "example.com" into RFC 1035 label format:
/// `\x07example\x03com\x00`
pub fn encode_qname(domain: &str) -> Result<Vec<u8>, DnsError> {
    if domain.is_empty() || domain.len() > 253 {
        return Err(DnsError::InvalidDomainName(domain.to_string()));
    }

    let mut qname = Vec::with_capacity(domain.len() + 2);

    for label in domain.split('.') {
        if label.is_empty() {
            return Err(DnsError::InvalidDomainName(domain.to_string()));
        }
        if label.len() > 63 {
            return Err(DnsError::InvalidDomainName(format!(
                "Label '{label}' exceeds 63 bytes"
            )));
        }

        qname.push(label.len() as u8);
        qname.extend_from_slice(label.as_bytes());
    }

    // Terminating null byte
    qname.push(0x00);
    Ok(qname)
}

/// Builds a complete DNS query packet for an A record (IPv4).
pub fn build_query_packet(domain: &str, tx_id: u16) -> Result<Vec<u8>, DnsError> {
    let header = DnsHeader::new_query(tx_id);
    let qname = encode_qname(domain)?;

    let mut packet = Vec::with_capacity(HEADER_LEN + qname.len() + 4);
    packet.extend_from_slice(&header.to_bytes());
    packet.extend_from_slice(&qname);

    // QTYPE = 0x0001 (A record)
    packet.extend_from_slice(&1u16.to_be_bytes());
    // QCLASS = 0x0001 (IN - Internet)
    packet.extend_from_slice(&1u16.to_be_bytes());

    Ok(packet)
}

/// Helper that skips a domain name field in a DNS packet buffer.
/// Domain names can be a sequence of labels ending in 0x00, or end in a 2-byte pointer (0xC0XX).
fn skip_name(buf: &[u8], mut offset: usize) -> Result<usize, DnsError> {
    loop {
        if offset >= buf.len() {
            return Err(DnsError::BufferTooShort {
                expected: offset + 1,
                available: buf.len(),
            });
        }

        let len = buf[offset];
        if (len & 0xC0) == 0xC0 {
            // Pointer is 2 bytes long
            if offset + 2 > buf.len() {
                return Err(DnsError::BufferTooShort {
                    expected: offset + 2,
                    available: buf.len(),
                });
            }
            return Ok(offset + 2);
        } else if len == 0x00 {
            // Null-byte terminator
            return Ok(offset + 1);
        } else {
            let label_len = len as usize;
            offset += 1 + label_len;
        }
    }
}

/// Parses answers from the DNS response packet.
pub fn parse_answers(buf: &[u8], header: &DnsHeader) -> Result<Vec<DnsAnswer>, DnsError> {
    let mut offset = HEADER_LEN;

    // Skip all questions (QDCOUNT)
    for _ in 0..header.qdcount {
        offset = skip_name(buf, offset)?;
        // Each question ends with 2 bytes QTYPE + 2 bytes QCLASS = 4 bytes
        if offset + 4 > buf.len() {
            return Err(DnsError::BufferTooShort {
                expected: offset + 4,
                available: buf.len(),
            });
        }
        offset += 4;
    }

    let mut answers = Vec::new();

    // Parse answers (ANCOUNT)
    for _ in 0..header.ancount {
        offset = skip_name(buf, offset)?;

        // Need at least 10 bytes: TYPE (2B) + CLASS (2B) + TTL (4B) + RDLENGTH (2B)
        if offset + 10 > buf.len() {
            return Err(DnsError::BufferTooShort {
                expected: offset + 10,
                available: buf.len(),
            });
        }

        let rtype = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let _rclass = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
        let ttl = u32::from_be_bytes([
            buf[offset + 4],
            buf[offset + 5],
            buf[offset + 6],
            buf[offset + 7],
        ]);
        let rdlength = u16::from_be_bytes([buf[offset + 8], buf[offset + 9]]) as usize;
        offset += 10;

        if offset + rdlength > buf.len() {
            return Err(DnsError::BufferTooShort {
                expected: offset + rdlength,
                available: buf.len(),
            });
        }

        // TYPE == 1 is A record (IPv4 address, rdlength must be 4)
        if rtype == 1 && rdlength == 4 {
            let ip = Ipv4Addr::new(
                buf[offset],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            );
            answers.push(DnsAnswer { ip, ttl });
        }

        offset += rdlength;
    }

    if answers.is_empty() {
        return Err(DnsError::NoAnswers);
    }

    Ok(answers)
}
