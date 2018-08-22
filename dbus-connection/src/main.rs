extern crate dbus;

use dbus::{BusType, Connection, arg};

use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
    let c = Connection::get_private(BusType::System).unwrap();
    c.add_match("type='signal',sender='org.freedesktop.NetworkManager',interface='org.freedesktop.NetworkManager'").unwrap();
    for msg in c.incoming(100000) {
        println!("{:?}", msg);
        let get: Option<HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>> = msg.get1();
        println!("{:?}", get);
        if let Some(hashmap) = get {
            for (k, _) in hashmap {
                if k == "ActiveConnections" {
                    println!("Active connections changed");
                }
            }
        }
    }
}
