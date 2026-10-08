# DNS Query Resolver (RFC 1035)

A minimalist, zero-dependency DNS client in Rust that constructs raw DNS query packets over UDP, transmits them to upstream recursive resolvers, and parses IPv4 `A` record responses.

## Features

- **Zero External Dependencies**: Built purely with Rust's standard library (`std::net::UdpSocket`, `std::time::Duration`, `std::net::Ipv4Addr`).
- **RFC 1035 Compliance**: 
  - Standard 12-byte header serialization and big-endian network byte order.
  - Domain name QNAME encoding into length-prefixed label chunks.
  - Recursion Desired (`RD = 1`) query flag bit-packing.
  - Pointer decompression support (`0xC0 | offset`) for compressed answer labels.
- **Robust Error Handling**:
  - Distinguishes between network I/O timeouts, malformed domain strings, truncated buffers, and DNS protocol error responses (e.g., `RCODE 3 = NXDomain`).
  - Zero `.unwrap()` calls in parsing logic.

## Project Structure

```
dns_resolver/
├── src/
│   ├── main.rs       # CLI entry point
│   ├── lib.rs        # Library interface & module exports
│   ├── error.rs      # DnsError domain error definitions
│   ├── protocol.rs   # Binary packet builder & wire parsing
│   ├── resolver.rs   # UDP socket transport & query dispatch
│   └── tests.rs      # Unit tests & synthetic wire format assertions
└── Cargo.toml
```

## Usage

### Run Unit Tests
```bash
cargo test
```

### Query a Domain
```bash
cargo run -- <domain> [dns_server_ip:port]
```

#### Example 1: Resolve `example.com` via Google DNS
```bash
cargo run -- example.com 8.8.8.8:53
```

**Output**:
```text
Querying DNS server [8.8.8.8:53] for 'example.com' (A record)...

Resolved 2 record(s) in 36.20ms:
  [1] IP: 104.20.23.154    (TTL: 300s)
  [2] IP: 172.66.147.243   (TTL: 300s)
```

#### Example 2: Non-existent Domain (NXDomain)
```bash
cargo run -- nonexistent-domain-123456789.xyz 1.1.1.1:53
```

**Output**:
```text
Querying DNS server [1.1.1.1:53] for 'nonexistent-domain-123456789.xyz' (A record)...

Query failed: DNS server error RCODE 3 (NXDomain)
```
