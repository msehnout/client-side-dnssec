use dbus::{BusType, Connection, Path, arg};
use dbus::stdintf::org_freedesktop_dbus::Properties;
use dbus::arg::RefArg;
use std::collections::HashMap;
use std::str::FromStr;
use std::net::Ipv4Addr;

use super::{ConnectionType, Connections, NetworkMonitor};

#[derive(PartialEq, Eq)]
enum MonitorState {
    Watching,
    Debouncing,
}

impl MonitorState {

    fn new() -> Self {
        MonitorState::Watching
    }

    fn next_timeout(&self) -> u32 {
        use self::MonitorState::*;
        match self {
            Watching => 1000,
            Debouncing => 20,
        }
    }
}

pub struct DbusMonitor {
    state: MonitorState,
    connection: Connection,
}

fn reverse_u32_order(i: u32) -> u32 {
    (
    ((i & 0xff000000) >> 24) |
    ((i & 0x00ff0000) >>  8) |
    ((i & 0x0000ff00) <<  8) |
    ((i & 0x000000ff) << 24)
    )
}

impl DbusMonitor {
    pub fn new() -> DbusMonitor {
        let connection = Connection::get_private(BusType::System).unwrap();
        let state = MonitorState::new();
        connection.add_match("type='signal',sender='org.freedesktop.NetworkManager',interface='org.freedesktop.NetworkManager'").unwrap();

        DbusMonitor {
            state,
            connection
        }
    }

    fn query_network_manager(&self, numbers: Vec<u32>) -> Connections {
        // So why on earth am I using the filter_map ??
        // Well as it turns out, when a connection is removed Network Manager sends a signal with a
        // list of old connections. So one of them will fail to inspect.
        use std::{thread, time};

        thread::sleep(time::Duration::from_millis(1500));
        let con = numbers.iter().filter_map(|i| {
            let p = self.connection.with_path("org.freedesktop.NetworkManager",
                                              format!("/org/freedesktop/NetworkManager/ActiveConnection/{}", i),
                                              5000);
            //println!("{:?}", p);
            // Query the active connection object
            let interface_name = "org.freedesktop.NetworkManager.Connection.Active";

            let id: String       = p.get(interface_name, "Id").ok()?;
            let con_type: String = p.get(interface_name, "Type").ok()?;
            let default: bool    = p.get(interface_name, "Default").ok()?;

            let con_type = ConnectionType::from_str(&con_type);

            // Query the IP4Config object
            let ipconfig_path: Path = p.get("org.freedesktop.NetworkManager.Connection.Active",
                                            "Ip4Config").ok()?;
            //println!("{:?}", ipconfig_path);
            let p = self.connection.with_path("org.freedesktop.NetworkManager", ipconfig_path, 5000);

            let interface_name = "org.freedesktop.NetworkManager.IP4Config";
            let mut domains: Vec<String> = p.get(interface_name, "Domains").ok()?;
            let searches: Vec<String>    = p.get(interface_name, "Searches").ok()?;
            let nameservers: Vec<u32>    = p.get(interface_name, "Nameservers").ok()?;
            let addresses: Vec<Vec<u32>>    = p.get(interface_name, "Addresses").ok()?;

            let nameservers: Vec<Ipv4Addr> = nameservers
                .into_iter()
                .map(|i| Ipv4Addr::from(reverse_u32_order(i)))
                .collect();
            domains.extend(searches);
            let addresses: Vec<(Ipv4Addr, u8)> = addresses
                .into_iter()
                .map(|i| (Ipv4Addr::from(reverse_u32_order(i[0])), i[1] as u8))
                .collect();

            //trace!("{:?}", domains);
            Some(super::Connection {
                id,
                domains,
                con_type,
                default,
                nameservers,
                addresses,
            })
        }).collect();

        Connections {
            con,
        }
    }
}

impl NetworkMonitor for DbusMonitor {
    fn wait_for_connections(&mut self) -> Connections {
        let mut get: Option<HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>> = None;
        loop {
            if let Some(msg) = self.connection.incoming(self.state.next_timeout()).next() {
                self.state = MonitorState::Watching;
                trace!("{:?}", msg);
                get = msg.get1();
                println!("{:?}", get);
                if let Some(hashmap) = &get {
                    if hashmap.contains_key("ActiveConnections") {
                        trace!("Active connections changed");
                        trace!("Debouncing");
                        self.state = MonitorState::Debouncing;
                    }
                }
            } else {
                if self.state == MonitorState::Debouncing {
                    trace!("Run update");

                    // WARNING: read this code on your own risk!
                    // It converts dynamically typed D-Bus data into statically typed Rust
                    // variables, it is very ugly and your head might explode while reading it!

                    // ... but it works ...

                    let maybe_numbers: Option<Vec<u32>> = get
                        // Get active connections field
                        // The closure returns Option<Value>
                        .and_then(|mut hashmap| {
                            // In order to obtain ownership of the value, I need to remove it
                            // from the collection
                            hashmap.remove_entry("ActiveConnections")
                                .map(|x| x.1)
                        })
                        // This closure takes the value from the collection and returns
                        // Option<Vec<u32>, therefore and_then
                        .and_then(|value| -> Option<Vec<u32>> {
                            // unwrap the Variant type (D-Bus defined)
                            // this returns Option<sth>
                            value.as_iter()
                                // so map over the optional value, therefore the closure must
                                // return the final vector
                                .map(|variant| -> Vec<u32> {
                                    // Now I have a list of sth, where the items can contains lists
                                    // of object paths. Don't ask me why, it just is in this format.
                                    variant.filter_map(|i| -> Option<Vec<u32>> {
                                        // Convert the i value into a list if possible, so this will
                                        // return Option<...>
                                        i.as_iter()
                                            // map over the Option
                                            .map(|list| {
                                                // This indeed was a list so we want to iterate
                                                // over it
                                                list.filter_map(|maybe_objpath| {
                                                    // Try to convert the items into an object path
                                                    maybe_objpath.as_str()
                                                        // if the conversion was successful, try to
                                                        // process it. The processing itself can
                                                        // return an error, so use and_then method
                                                        // instead of simple map.
                                                        .and_then(|objpath| -> Option<u32> {
                                                            let path: Vec<&str> = objpath.split("/").collect();
                                                            u32::from_str(path[path.len()-1]).ok()
                                                        })
                                                }).collect()
                                            })
                                    }).flatten().collect()
                                })
                        });
                    println!("{:?}", maybe_numbers);
                    get = None;

                    if let Some(numbers) = maybe_numbers {
                        self.state = MonitorState::Watching;
                        return self.query_network_manager(numbers);
                    }
                }
            }
        }
    }
}

#[test]
fn test_reverse_u32_order() {
    let input = 0xAABBCCDD;
    assert_eq!(reverse_u32_order(input), 0xDDCCBBAA);
}