# Phase 2: Error Handling & Data Processing

This phase explores idiomatic error handling in Rust, binary encoding/decoding, network protocols, and robust stream parsing.

## Projects in this Phase

1. **Custom Binary Frame Decoder**
   - **Focus**: Working with byte slices, header framing, payload bounds validation, endianness handling, and custom domain error hierarchies (`std::error::Error` / `thiserror`).

2. **[NEW] DNS Query Resolver**
   - **Focus**: Network socket programming with UDP (`UdpSocket`), building raw RFC-compliant DNS wire frames, parsing DNS headers, questions, and resource records with error handling.
