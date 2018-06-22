use failure::Error;

use serde_json;

use std::net::Ipv4Addr;
use std::collections::HashMap;

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
pub enum ConnectionType {
    Ethernet = 0,
    VPN = 1,
    WiFi = 2,
    Other = 3,
}

pub type Domain = String;

/// Strongly typed connection
#[derive(Debug)]
pub struct Connection {
    id: String,
    con_type: ConnectionType,
    default: bool,
    addresses: Vec<(Ipv4Addr, u8)>,
    nameservers: Vec<Ipv4Addr>,
    domains: Vec<Domain>,
}

#[derive(Debug)]
pub struct ForwardZone {
    pub domain: Domain,
    pub nameservers: Vec<Ipv4Addr>,
    pub con_type: ConnectionType,
}


#[derive(Debug)]
pub struct ReverseZone {
    pub zone: &'static str,
    pub nameservers: Vec<Ipv4Addr>,
    pub con_type: ConnectionType,
}

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

pub fn parse_connections(input: &str) -> Result<Vec<Connection>, Error> {
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

pub fn get_forward_zones(connections: &Vec<Connection>) -> Vec<ForwardZone> {
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

pub fn get_reverse_zones(connections: &Vec<Connection>) -> Vec<ReverseZone> {
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

//static TESTING_INPUT: &'static str = r#"[{"id": "enp0s25", "type": "802-3-ethernet", "default": true, "addresses": ["10.10.0.10/24"], "nameservers": ["10.10.0.99", "10.10.0.88"], "domains": ["afk.redhat.com", "redhat.com"]}, {"id": "Red Hat WIFI", "type": "802-11-wireless", "default": false, "addresses": ["10.111.111.111/21"], "nameservers": ["10.111.111.110", "10.111.111.109", "10.111.111.108"], "domains": ["redhat.com"]}, {"id": "Red Hat VPN", "type": "vpn", "default": false, "addresses": ["10.11.111.111/22"], "nameservers": ["10.11.111.10", "10.11.111.11"], "domains": ["redhat.com"]}, {"id": "tun0", "type": "tun", "default": false, "addresses": ["10.40.0.6/22"], "nameservers": [], "domains": []}]"#;

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

