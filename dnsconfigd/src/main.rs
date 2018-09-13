//! # DNS Configuration daemon
//!
//! Works roughly like this:
//! ```
//! Monitor --> Calculator --> Backend
//! ```
//! where each stage is defined by a trait, run in a separate thread and connected using channels.

#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate dbus;
#[macro_use]
extern crate log;
extern crate env_logger;

use docopt::Docopt;

mod dbus_monitor;
mod connection;

pub use connection::*;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
dnsconfigd - Dynamic DNS configuration daemon

Usage:
  dnsconfigd
  dnsconfigd (-h | --help)
  dnsconfigd --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_help: bool,
    flag_version: bool,
}

/// Output of the first stage (monitor)
/// So far this is just a wrapper around vector of Connection structures. Mainly to make the
/// function signatures look nice.
#[derive(Debug)]
pub struct Connections {
    con: Vec<Connection>,
}

/// This is the output of the second stage (calculator) defining how to set up the backend DNS
/// resolver.
pub struct SplitView {}

/// # 1st stage
///
/// An entity that monitors the network configuration and reports any changes. The example
/// implementation is available in dbus_monitor which, as the name suggests, implements monitoring
/// of Network Manager over D-Bus.
pub trait NetworkMonitor {
    fn wait_for_connections(&mut self) -> Connections;
}

/// # 2nd stage
pub trait SplitViewCalculator {
    fn calc(connections: Connections) -> SplitView;
}

/// # 3rd stage
pub trait Backend {
    fn set(view: SplitView) -> Result<(), ()>;
}

fn run<M/*,C,B*/>(mut monitor: M/*, calc: C, backend: B*/) -> Result<(), &'static str>
    where M: NetworkMonitor,
        // C: SplitViewCalculator,
        // B: Backend
{
    loop {
        let connections = monitor.wait_for_connections();
        info!("New connections: {:#?}", connections);
    }
}

fn main() {
    env_logger::init();

    let _args: Args = Docopt::new(USAGE)
        .and_then(|d| {
            d.help(true)
                .version(VERSION.map(|s| s.to_string()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());

    info!("Running the daemon");

    let monitor = dbus_monitor::DbusMonitor::new();

    if let Err(e) = run(monitor) {
        error!("Failed with {:?}", e);
    }
}
