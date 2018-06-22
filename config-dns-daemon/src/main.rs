extern crate docopt;
extern crate env_logger;
extern crate failure;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use docopt::Docopt;
use env_logger::{Builder, WriteStyle};
use failure::Error;
use log::LevelFilter;

use std::io::prelude::*;
use std::io::BufReader;
use std::os::unix::net::{UnixStream, UnixListener};

mod connections;
mod knot_backend;

use connections::*;
use knot_backend::*;

#[cfg(test)]
const BINARY_NAME: &'static str = "config-dns-daemon";
const USAGE: &'static str = "
Daemon for automatic DNS resolver configuration
triggered by changes in network setup.

Usage:
  config-dns-daemon [--verbosity=<level> | --socket=<path>]
  config-dns-daemon (-h | --help)
  config-dns-daemon (-v | --version)

Options:
  -h, --help            Show this screen.
  --socket=<path>      Path to the Unix domain socket used for IPC with control script
  --verbosity=<level>  Level of verbosity (TODO range).
  -v, --version         Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_help: bool,
    flag_socket: Option<String>,
    flag_verbosity: Option<String>,
    flag_version: bool,
}

fn handle_control_connection(stream: UnixStream) -> Result<String, Error> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let len = reader.read_line(&mut line)?;
    info!("First line is {} bytes long and contains {:#?}", len, line);
    writer.write_all(b"Success")?;
    Ok(line)
}

fn run_control_socket(socket_path: &str) -> Result<(), Error> {
    info!("Removing socket at path: {}", socket_path);
    std::fs::remove_file(socket_path)?;
    info!("Starting socket at path: {}", socket_path);
    let listener = UnixListener::bind(socket_path)?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                /* connection succeeded */
                info!("Connection established! Reading until the end of line.");
                match handle_control_connection(stream) {
                    Ok(line) => {
                        if let Ok(connections) = parse_connections(&line) {
                            let fwd_zones = get_forward_zones(&connections);
                            let reverse_zones = get_reverse_zones(&connections);
                            info!("Forward zones: {:?}", fwd_zones);
                            info!("Reverse zones: {:?}", reverse_zones);
                            if let Err(e) = apply_rules(fwd_zones, reverse_zones){
                                error!("Failed to apply forwarding rules to the resolver: {}", e)
                            }
                        } else {
                            error!("Could not parse control input.");
                        }
                    },
                    Err(e) => error!("Stream returned an error: {}", e),
                }
            }
            Err(_err) => {
                /* connection failed */
                error!("Connection failed! Trying the next one")
            }
        }
    }
    std::fs::remove_file(socket_path)?;
    Ok(())
}

fn main() {
    let mut builder = Builder::new();

    builder.filter(None, LevelFilter::Info)
        .write_style(WriteStyle::Always)
        .init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(std::env::args()).deserialize())
        .unwrap_or_else(|e| e.exit());

    let socket_path = &(args.flag_socket).unwrap_or("./socket".to_string());
    if let Err(e) = run_control_socket(socket_path) {
        error!("Failed with {}", e);
    }
}

#[test]
fn parse_usage() {
    Docopt::new(USAGE).unwrap();
}

#[test]
fn parse_help_arg() {
    let argv = || vec![BINARY_NAME, "-h"];

    if let Err(_) = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv().into_iter()).parse()) {
        // fine
    } else {
        panic!("This should fail")
    }
}

#[test]
fn parse_verbosity_arg() {
    let argv = || vec![BINARY_NAME, "--verbosity=3"];

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(argv().into_iter()).deserialize())
        .unwrap();
}