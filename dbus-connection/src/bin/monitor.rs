extern crate dbus;

use dbus::{BusType, Connection, arg};

use std::collections::HashMap;

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
        use MonitorState::*;
        match self {
            Watching => 1000,
            Debouncing => 20,
        }
    }
}

fn main() {
    println!("Hello, world!");
    let c = Connection::get_private(BusType::System).unwrap();
    let mut state = MonitorState::new();
    c.add_match("type='signal',sender='org.freedesktop.NetworkManager',interface='org.freedesktop.NetworkManager'").unwrap();
    loop {
        if let Some(msg) = c.incoming(state.next_timeout()).next() {
            state = MonitorState::Watching;
            println!("{:?}", msg);
            let get: Option<HashMap<String, arg::Variant<Box<arg::RefArg + 'static>>>> = msg.get1();
            println!("{:?}", get);
            if let Some(hashmap) = get {
                for (k, _) in hashmap {
                    if k == "ActiveConnections" {
                        println!("Active connections changed");
                        println!("Debouncing");
                        state = MonitorState::Debouncing;
                    }
                }
            }
        } else {
            if state == MonitorState::Debouncing {
                println!("Run update");
                state = MonitorState::Watching;
            }
        }
    }
}
