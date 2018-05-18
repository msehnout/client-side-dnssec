extern crate trust_dns_resolver;

use std::net::*;
use std::env;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol};

fn main() {
    let mut args = env::args();
    if args.len() < 4 {
        println!("Usage: resolver <forward domain> <ip of dns resolver> <query>")
    }
    args.next();
    let forward_domain = args.next().unwrap();
    let resolver_ip = args.next().unwrap();
    let query = args.next().unwrap();

    println!("Run testing resolver with split on domain {} with DNS resolver {}:", forward_domain, resolver_ip);

    if query.ends_with(&forward_domain) {
        let mut config = ResolverConfig::default();
        let nm_config = NameServerConfig {
            socket_addr: SocketAddr::new(IpAddr::V4(resolver_ip.parse().unwrap()), 53),
            protocol: Protocol::Udp,
            tls_dns_name: None,
        };
        config.add_name_server(nm_config);
        let options = ResolverOpts::default();
        let resolver = Resolver::new(config, options).unwrap();
        let result = resolver.lookup_ip(&query);
        println!("Forward resolver result: {:?}", result);
    } else {
        let resolver = Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default()).unwrap();
        let result = resolver.lookup_ip(&query);
        println!("Global resolver result: {:?}", result);
    }
}
