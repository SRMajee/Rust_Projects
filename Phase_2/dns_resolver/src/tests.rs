use crate::protocol::{build_query_packet, encode_qname, parse_answers, DnsHeader};
use std::net::Ipv4Addr;

#[test]
fn test_qname_encoding_standard() {
    let encoded = encode_qname("google.com").unwrap();
    // 6 "google" 3 "com" 0
    let expected = b"\x06google\x03com\x00";
    assert_eq!(encoded, expected);
}

#[test]
fn test_qname_encoding_multi_level() {
    let encoded = encode_qname("sub.domain.co.uk").unwrap();
    let expected = b"\x03sub\x06domain\x02co\x02uk\x00";
    assert_eq!(encoded, expected);
}

#[test]
fn test_qname_empty_or_malformed() {
    assert!(encode_qname("").is_err());
    assert!(encode_qname(".google.com").is_err());
    assert!(encode_qname("google..com").is_err());
}

#[test]
fn test_header_serialization_and_parsing() {
    let header = DnsHeader::new_query(0x1234);
    let bytes = header.to_bytes();

    assert_eq!(&bytes[0..2], &0x1234u16.to_be_bytes());
    assert_eq!(&bytes[2..4], &0x0100u16.to_be_bytes()); // RD set
    assert_eq!(&bytes[4..6], &1u16.to_be_bytes()); // QDCOUNT = 1

    let parsed = DnsHeader::parse(&bytes).unwrap();
    assert_eq!(parsed.id, 0x1234);
    assert_eq!(parsed.flags, 0x0100);
    assert_eq!(parsed.qdcount, 1);
    assert_eq!(parsed.ancount, 0);
}

#[test]
fn test_query_packet_construction() {
    let packet = build_query_packet("example.com", 0xABCD).unwrap();
    // Header is 12 bytes
    assert_eq!(&packet[0..2], &0xABCDu16.to_be_bytes());

    // QNAME starts at offset 12
    let qname = &packet[12..12 + 13]; // 7 + "example" + 3 + "com" + 0 = 13 bytes
    assert_eq!(qname, b"\x07example\x03com\x00");

    // QTYPE = 1 (A), QCLASS = 1 (IN)
    let trailer = &packet[12 + 13..];
    assert_eq!(trailer, &[0x00, 0x01, 0x00, 0x01]);
}

#[test]
fn test_parse_response_with_pointer_compression() {
    // Synthetic DNS response:
    // Header (12 bytes): ID=0x1234, Flags=0x8180 (standard response, no error), QD=1, AN=1, NS=0, AR=0
    let mut resp = Vec::new();
    resp.extend_from_slice(&0x1234u16.to_be_bytes());
    resp.extend_from_slice(&0x8180u16.to_be_bytes()); // Standard response, No error
    resp.extend_from_slice(&1u16.to_be_bytes()); // QDCOUNT
    resp.extend_from_slice(&1u16.to_be_bytes()); // ANCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // NSCOUNT
    resp.extend_from_slice(&0u16.to_be_bytes()); // ARCOUNT

    // Question: \x03foo\x03bar\x00, QTYPE=1, QCLASS=1
    resp.extend_from_slice(b"\x03foo\x03bar\x00");
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());

    // Answer: Name pointer pointing to offset 12 (0xC00C)
    resp.extend_from_slice(&[0xC0, 0x0C]);
    // TYPE = 1 (A), CLASS = 1 (IN), TTL = 300, RDLENGTH = 4, RDATA = 93.184.216.34
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&1u16.to_be_bytes());
    resp.extend_from_slice(&300u32.to_be_bytes());
    resp.extend_from_slice(&4u16.to_be_bytes());
    resp.extend_from_slice(&[93, 184, 216, 34]);

    let header = DnsHeader::parse(&resp).unwrap();
    let answers = parse_answers(&resp, &header).unwrap();

    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].ip, Ipv4Addr::new(93, 184, 216, 34));
    assert_eq!(answers[0].ttl, 300);
}
