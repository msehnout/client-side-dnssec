use std::net::Ipv4Addr;

/// Type of a connection as reported by Network Manager
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConnectionType {
    Ethernet = 0,
    VPN = 1,
    WiFi = 2,
    Other = 3,
}

impl ConnectionType {
    pub fn from_str(s: &str) -> Self {
        if s.contains("ethernet") {
            ConnectionType::Ethernet
        } else if s.contains("wireless") {
            ConnectionType::WiFi
        } else if s.contains("vpn") {
            ConnectionType::VPN
        } else {
            ConnectionType::Other
        }
    }
}

pub type Domain = String;

/// Structure containing all information, that are relevant for DNS configuration about each
/// connection.
#[derive(Debug)]
pub struct Connection {
    pub id: String,
    pub con_type: ConnectionType,
    pub default: bool,
    pub addresses: Vec<(Ipv4Addr, u8)>,
    pub nameservers: Vec<Ipv4Addr>,
    pub domains: Vec<Domain>,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            id: "".to_string(),
            con_type: ConnectionType::Other,
            default: false,
            addresses: vec![],
            nameservers: vec![],
            domains: vec![],
        }
    }
}
