use crate::error::FrameError;
use crate::frame::{compute_checksum, Frame, MessageType, HEADER_LEN, MAGIC_BYTES};

#[test]
fn test_decode_valid_ping_frame_empty_payload() {
    let frame = Frame::new(1, MessageType::Ping, &[]);
    let encoded = frame.encode();

    let (decoded, remaining) = Frame::decode(&encoded).expect("failed to decode valid ping");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.msg_type, MessageType::Ping);
    assert_eq!(decoded.payload, b"");
    assert!(remaining.is_empty());
}

#[test]
fn test_decode_valid_data_frame() {
    let payload = b"Hello, Binary Network Protocol!";
    let frame = Frame::new(1, MessageType::Data, payload);
    let encoded = frame.encode();

    let (decoded, remaining) = Frame::decode(&encoded).expect("failed to decode valid data frame");
    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.msg_type, MessageType::Data);
    assert_eq!(decoded.payload, payload);
    assert!(remaining.is_empty());
}

#[test]
fn test_stream_parsing_multiple_consecutive_frames() {
    let frame1 = Frame::new(1, MessageType::Ping, b"ping-1");
    let frame2 = Frame::new(1, MessageType::Data, b"data-2");
    let frame3 = Frame::new(1, MessageType::Ack, b"ack-3");

    let mut stream = Vec::new();
    stream.extend_from_slice(&frame1.encode());
    stream.extend_from_slice(&frame2.encode());
    stream.extend_from_slice(&frame3.encode());

    // Decode frame 1
    let (d1, rest1) = Frame::decode(&stream).expect("frame 1 failed");
    assert_eq!(d1.payload, b"ping-1");
    assert_eq!(d1.msg_type, MessageType::Ping);

    // Decode frame 2
    let (d2, rest2) = Frame::decode(rest1).expect("frame 2 failed");
    assert_eq!(d2.payload, b"data-2");
    assert_eq!(d2.msg_type, MessageType::Data);

    // Decode frame 3
    let (d3, rest3) = Frame::decode(rest2).expect("frame 3 failed");
    assert_eq!(d3.payload, b"ack-3");
    assert_eq!(d3.msg_type, MessageType::Ack);

    assert!(rest3.is_empty());
}

#[test]
fn test_error_incomplete_header() {
    let truncated = [0xAA, 0x55, 0x01]; // only 3 bytes instead of 6
    let err = Frame::decode(&truncated).unwrap_err();
    assert_eq!(
        err,
        FrameError::IncompleteHeader {
            expected: HEADER_LEN,
            actual: 3,
        }
    );
}

#[test]
fn test_error_invalid_magic() {
    let mut bad_magic = Frame::new(1, MessageType::Ping, &[]).encode();
    bad_magic[0] = 0xBE;
    bad_magic[1] = 0xEF;

    let err = Frame::decode(&bad_magic).unwrap_err();
    assert_eq!(
        err,
        FrameError::InvalidMagic {
            expected: MAGIC_BYTES,
            found: [0xBE, 0xEF],
        }
    );
}

#[test]
fn test_error_unsupported_version() {
    let mut bad_version = Frame::new(1, MessageType::Ping, &[]).encode();
    bad_version[2] = 2; // version 2 unsupported

    // Recompute checksum so only version fails
    let checksum = compute_checksum(&bad_version[..bad_version.len() - 1]);
    let last = bad_version.len() - 1;
    bad_version[last] = checksum;

    let err = Frame::decode(&bad_version).unwrap_err();
    assert_eq!(err, FrameError::UnsupportedVersion(2));
}

#[test]
fn test_error_unknown_message_type() {
    let mut bad_type = Frame::new(1, MessageType::Ping, &[]).encode();
    bad_type[3] = 0xFF; // opcode 0xFF

    let checksum = compute_checksum(&bad_type[..bad_type.len() - 1]);
    let last = bad_type.len() - 1;
    bad_type[last] = checksum;

    let err = Frame::decode(&bad_type).unwrap_err();
    assert_eq!(err, FrameError::UnknownMessageType(0xFF));
}

#[test]
fn test_error_incomplete_payload() {
    let mut buffer = Frame::new(1, MessageType::Data, b"1234567890").encode();
    // Slice off trailing 3 bytes
    buffer.truncate(buffer.len() - 3);

    let err = Frame::decode(&buffer).unwrap_err();
    match err {
        FrameError::IncompletePayload { expected, actual } => {
            assert_eq!(expected, 10 + 1); // 10 bytes payload + 1 byte checksum
            assert_eq!(actual, buffer.len() - HEADER_LEN);
        }
        other => panic!("expected IncompletePayload, got {other:?}"),
    }
}

#[test]
fn test_error_payload_too_large() {
    // Manually craft header claiming 2000 bytes payload (> MAX_PAYLOAD_LEN 1024)
    let mut buf = vec![0xAA, 0x55, 0x01, 0x02];
    buf.extend_from_slice(&2000u16.to_be_bytes());
    buf.extend(vec![0u8; 10]); // incomplete anyway, but size check triggers first

    let err = Frame::decode(&buf).unwrap_err();
    assert_eq!(
        err,
        FrameError::PayloadTooLarge {
            max: 1024,
            actual: 2000,
        }
    );
}

#[test]
fn test_error_checksum_mismatch() {
    let mut corrupted = Frame::new(1, MessageType::Data, b"secure payload").encode();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 0xFF; // corrupt checksum byte

    let err = Frame::decode(&corrupted).unwrap_err();
    match err {
        FrameError::ChecksumMismatch { .. } => {}
        other => panic!("expected ChecksumMismatch, got {other:?}"),
    }
}

#[test]
fn test_error_display_formatting() {
    let err = FrameError::InvalidMagic {
        expected: [0xAA, 0x55],
        found: [0x12, 0x34],
    };
    let formatted = format!("{err}");
    assert!(formatted.contains("Invalid magic sync bytes"));
}
