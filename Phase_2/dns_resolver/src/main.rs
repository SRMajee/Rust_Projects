use dns_resolver::{DnsError, DnsResolver};
use std::env;
use std::net::SocketAddr;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: dns_resolver <domain> [dns_server_ip:port]");
        eprintln!("Example: dns_resolver google.com 8.8.8.8:53");
        std::process::exit(1);
    }

    let domain = &args[1];
    let server_addr: SocketAddr = if args.len() >= 3 {
        let addr_str = if args[2].contains(':') {
            args[2].clone()
        } else {
            format!("{}:53", args[2])
        };
        addr_str.parse()?
    } else {
        "8.8.8.8:53".parse()?
    };

    println!("Querying DNS server [{server_addr}] for '{domain}' (A record)...");

    let resolver = DnsResolver::new(server_addr);
    let start = Instant::now();

    match resolver.resolve_a(domain) {
        Ok(answers) => {
            let duration = start.elapsed();
            println!("\nResolved {} record(s) in {:.2?}:", answers.len(), duration);
            for (idx, ans) in answers.iter().enumerate() {
                println!("  [{}] IP: {:<16} (TTL: {}s)", idx + 1, ans.ip, ans.ttl);
            }
        }
        Err(DnsError::ServerFailure(rcode)) => {
            eprintln!("\nQuery failed: DNS server error RCODE {rcode}");
            std::process::exit(2);
        }
        Err(err) => {
            eprintln!("\nResolution error: {err}");
            std::process::exit(1);
        }
    }

    Ok(())
}
