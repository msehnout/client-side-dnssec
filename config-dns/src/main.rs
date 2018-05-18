extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::net::Ipv4Addr;
use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::io::BufReader;
use std::env;

use failure::Error;

static TESTING_INPUT: &'static str = r#"[{"id": "enp0s25", "type": "802-3-ethernet", "default": true, "addresses": ["10.10.0.10/24"], "nameservers": ["10.10.0.99", "10.10.0.88"], "domains": ["afk.redhat.com", "redhat.com"]}, {"id": "Red Hat WIFI", "type": "802-11-wireless", "default": false, "addresses": ["10.111.111.111/21"], "nameservers": ["10.111.111.110", "10.111.111.109", "10.111.111.108"], "domains": ["redhat.com"]}, {"id": "Red Hat VPN", "type": "vpn", "default": false, "addresses": ["10.11.111.111/22"], "nameservers": ["10.11.111.10", "10.11.111.11"], "domains": ["redhat.com"]}, {"id": "tun0", "type": "tun", "default": false, "addresses": ["10.40.0.6/22"], "nameservers": [], "domains": []}]"#;

/// Weakly typed connection
/// This should eventually go away
#[derive(Debug, Serialize, Deserialize)]
struct ConnectionWeak {
    id: String,
    #[serde(rename = "type")]
    con_type: String,
    default: bool,
    addresses: Vec<String>,
    nameservers: Vec<String>,
    domains: Vec<String>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum ConnectionType {
    Ethernet = 0,
    VPN = 1,
    WiFi = 2,
    Other = 3,
}

type Domain = String;

/// Strongly typed connection
#[derive(Debug)]
struct Connection {
    id: String,
    con_type: ConnectionType,
    default: bool,
    addresses: Vec<(Ipv4Addr, u8)>,
    nameservers: Vec<Ipv4Addr>,
    domains: Vec<Domain>,
}

#[derive(Debug)]
struct ForwardZone {
    domain: Domain,
    nameservers: Vec<Ipv4Addr>,
    con_type: ConnectionType,
}


#[derive(Debug)]
struct ReverseZone {
    zone: &'static str,
    nameservers: Vec<Ipv4Addr>,
    con_type: ConnectionType,
}


static RFC1918_REVERSE_ZONES: [&'static str; 20] =
    ["c.f.ip6.arpa",
    "d.f.ip6.arpa",
    "168.192.in-addr.arpa",
    "16.172.in-addr.arpa",
    "17.172.in-addr.arpa",
    "18.172.in-addr.arpa",
    "19.172.in-addr.arpa",
    "20.172.in-addr.arpa",
    "21.172.in-addr.arpa",
    "22.172.in-addr.arpa",
    "23.172.in-addr.arpa",
    "24.172.in-addr.arpa",
    "25.172.in-addr.arpa",
    "26.172.in-addr.arpa",
    "27.172.in-addr.arpa",
    "28.172.in-addr.arpa",
    "29.172.in-addr.arpa",
    "30.172.in-addr.arpa",
    "31.172.in-addr.arpa",
    "10.in-addr.arpa"];

fn ipv4_to_reverse_zone(addr: &Ipv4Addr) -> Option<&'static str> {
    let octets = addr.octets();
    match octets[0] {
        10 => Some("10.in-addr.arpa"),
        172 => Some("31.172.in-addr.arpa"), // TODO
        192 => {
            match octets[1] {
                168 => Some("168.192.in-addr.arpa"),
                _ => None,
            }
        },
        _ => None
    }
}

fn parse_connections(input: &str) -> Result<Vec<Connection>, Error> {
    let connections: Vec<ConnectionWeak> = serde_json::from_str(input)?;
    let connections: Vec<Connection> = connections.into_iter()
        .filter_map(|c| {
            let id = c.id;
            let con_type = if c.con_type.contains("ethernet") { ConnectionType::Ethernet }
                else if c.con_type.contains("wireless") { ConnectionType::WiFi }
                    else if c.con_type.contains("vpn") { ConnectionType::VPN }
                        else { ConnectionType::Other };
            let default = c.default;
            let addresses: Vec<(Ipv4Addr, u8)> = c.addresses.iter()
                .filter_map(|a| {
                    let split: Vec<&str> = a.split('/').collect();
                    if split.len() < 2 {
                        return None
                    }
                    let addr = if let Ok(a) = split[0].parse::<Ipv4Addr>() {a} else {return None};
                    let prefix = if let Ok(a) = split[1].parse::<u8>() {a} else {return None};
                    Some((addr, prefix))
                })
                .collect();
            let nameservers = c.nameservers.iter()
                .filter_map(|a| a.parse::<Ipv4Addr>().ok() )
                .collect();
            let domains = c.domains;
            Some(Connection {
                id, con_type, default, addresses, nameservers, domains
            })
        })
        .collect();
    Ok(connections)
}

fn get_forward_zones(connections: &Vec<Connection>) -> Vec<ForwardZone> {
    let forward_zones: Vec<ForwardZone> = connections.iter()
        .filter(|c| c.domains.len() != 0)
        .flat_map(|c| {
            c.domains.iter().map(move |d| {
                ForwardZone {
                    domain: d.clone(),
                    nameservers: c.nameservers.clone(),
                    con_type: c.con_type,
                }
            })
        })
        .collect();

    let indexes;
    {
        let mut forward_zones_unique: HashMap<&String, (usize, &ForwardZone)> = HashMap::new();
        for (i, zone) in forward_zones.iter().enumerate() {
            let insert;
            {
                let fwd_zone = forward_zones_unique.entry(&zone.domain).or_insert((i, zone));
                insert = fwd_zone.1.con_type > zone.con_type;
            }
            if insert {
                forward_zones_unique.insert(&zone.domain, (i, zone));
            }
        }

        indexes = forward_zones_unique.into_iter()
            .map(|(_, (i, _))| i)
            .collect::<Vec<_>>();
    }

    forward_zones.into_iter()
        .enumerate()
        .filter(|(i, _)| indexes.contains(&i))
        .map(|(_, zone)| zone)
        .collect()
}

fn get_reverse_zones(connections: &Vec<Connection>) -> Vec<ReverseZone> {
    let reverse_zones: Vec<ReverseZone> = connections.iter()
        .filter(|c| c.addresses.len() != 0)
        .flat_map(|c| {
            c.addresses.iter()
                .filter_map(move |a| {
                    Some(ReverseZone {
                        zone: if let Some(zone) = ipv4_to_reverse_zone(&a.0) {
                            zone
                        } else {
                            return None
                        },
                        nameservers: c.nameservers.clone(),
                        con_type: c.con_type,
                    })
                })
        })
        .collect();

    let indexes;
    {
        let mut reverse_zones_unique: HashMap<&str, (usize, &ReverseZone)> = HashMap::new();
        for (i, zone) in reverse_zones.iter().enumerate() {
            let insert;
            {
                let fwd_zone = reverse_zones_unique.entry(&zone.zone).or_insert((i, zone));
                insert = fwd_zone.1.con_type > zone.con_type;
            }
            if insert {
                reverse_zones_unique.insert(&zone.zone, (i, zone));
            }
        }

        indexes = reverse_zones_unique.into_iter()
            .map(|(_, (i, _))| i)
            .collect::<Vec<_>>();
    }

    reverse_zones.into_iter()
        .enumerate()
        .filter(|(i, _)| indexes.contains(&i))
        .map(|(_, zone)| zone)
        .collect()
}

fn read_response_from_socket<T: BufRead>(reader: &mut T) {
    let mut line;
    loop {
        line = String::new();
        let len = reader.read_line(&mut line).unwrap();
        print!("line [{}]:{}", len, line);
        if line == "\n" {
            break;
        }
    }

}

fn run() -> Result<(), Error> {
    let mut args =  env::args().skip(1);
    let connections;
    if let Some(arg) = args.next() {
        println!("Taking connections from cmd line");
        connections = parse_connections(&arg)?;
    } else {
        connections = parse_connections(TESTING_INPUT)?;
    }

    println!("{:?}", connections);

    let forward_zones_unique = get_forward_zones(&connections);
    println!("{:?}", forward_zones_unique);

    let reverse_zones_unique = get_reverse_zones(&connections);
    println!("{:?}", reverse_zones_unique);


    let mut stream = UnixStream::connect("/run/knot-resolver/control@1")?;
    for i in &forward_zones_unique {
        let policy_rule = format!("policy.add(policy.suffix(policy.STUB('{}'), {{todname('{}')}}))\n", i.nameservers[0], i.domain);
        print!("{}", policy_rule);
        stream.write_all(policy_rule.as_bytes())?;
        let mut reader = BufReader::new(&stream);
        read_response_from_socket(&mut reader);
    }
    for i in reverse_zones_unique {
        let policy_rule = format!("policy.add(policy.suffix(policy.STUB('{}'), {{todname('{}')}}))", i.nameservers[0], i.zone);
        print!("{}", policy_rule);
        stream.write_all(policy_rule.as_bytes())?;
        let mut reader = BufReader::new(&stream);
        read_response_from_socket(&mut reader);
    }

    let policy_rule =
        format!("policy.add(policy.all(policy.TLS_FORWARD({{{{'1.1.1.1', hostname='cloudflare-dns.com', ca_file='/etc/pki/tls/certs/ca-bundle.crt'}}}})))");
    print!("{}", policy_rule);
    stream.write_all(policy_rule.as_bytes())?;
    let mut reader = BufReader::new(&stream);
    read_response_from_socket(&mut reader);

    Ok(())
}

fn main() {
    println!("Hello, world!");
    if let Err(e) = run() {
        eprintln!("Error: {:?}", e);
    }
}

#[test]
fn reverse_zone_class_a() {
    let addr = Ipv4Addr::new(10, 20, 30, 40);
    assert_eq!(ipv4_to_reverse_zone(&addr), Some("10.in-addr.arpa"))
}

#[test]
fn reverse_zone_class_c_none() {
    let addr = Ipv4Addr::new(192, 120, 30, 40);
    assert_eq!(ipv4_to_reverse_zone(&addr), None)
}
