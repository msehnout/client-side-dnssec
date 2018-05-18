extern crate futures;
extern crate tokio_core;
extern crate trust_dns;
extern crate trust_dns_proto;
extern crate failure;
//#[macro_use] extern crate failure_derive;
#[macro_use] extern crate lazy_static;

use std::env;
use std::str::FromStr;

use trust_dns::client::{ClientConnection, ClientFuture};
use trust_dns::udp::UdpClientConnection;
use trust_dns::tcp::TcpClientConnection;
use trust_dns::rr::{Name, RecordType};
use trust_dns::op::{Edns, Message, Query};
use trust_dns::rr::rdata::opt::{EdnsOption, EdnsCode};

use trust_dns_proto::DnsHandle;

use failure::Error;
use futures::prelude::*;
use tokio_core::reactor::{Core};

lazy_static! {
    static ref TESTING_SERVER: Name = Name::from_str("dnssec-tools.org.")
                                            .expect("Name building should never fail.");
}

#[derive(Debug)]
pub enum TestResult {
    Success,
    Fail(&'static str),
}

/// [RFC 8027, section 3.1.1 and 3.1.2](https://tools.ietf.org/html/rfc8027#section-3.1.1)
pub fn support_simple_answers<DH>(dns_handle: &mut DH) -> impl Future<Item=TestResult, Error=DH::Error>
    where DH: DnsHandle
{
    dns_handle
        .lookup(Query::query(TESTING_SERVER.clone(), RecordType::A))
        .map(|_| TestResult::Success)
}

/// [RFC 8027, section 3.1.3](https://tools.ietf.org/html/rfc8027#section-3.1.3)
///
/// sth...
pub fn support_edns0<DH>(dns_handle: &mut DH) -> impl Future<Item=TestResult, Error=DH::Error>
    where DH: DnsHandle
{
    // Create a query
    let query = Query::query(TESTING_SERVER.clone(), RecordType::A);
    // Create an EDNS struct
    let mut edns = Edns::new();
    let v = vec![];
    edns.set_option(EdnsOption::from((EdnsCode::Zero, &v[..])));
    // Finally, assemble a message
    let mut msg = Message::new();
    msg.add_query(query);
    msg.set_edns(edns);

    dns_handle
        .send(msg)
        .map(|msg| {
            if let Some(edns) = msg.edns() {
                if edns.version() == 0 {
                    TestResult::Success
                } else {
                    TestResult::Fail("Wrong EDNS option")
                }
            } else {
                TestResult::Fail("No EDNS option")
            }
        })
}


/// [RFC 8027, section 3.1.4](https://tools.ietf.org/html/rfc8027#section-3.1.4)
///
/// This function implements [RFC 8027, section 3.1.4](https://tools.ietf.org/html/rfc8027#section-3.1.4)
/// which tests resolver for DO bit support. (DO stands for DNSSEC Ok and is defined in
/// [RFC 6891, section 6.1.4](https://tools.ietf.org/html/rfc6891#section-6.1.4).
pub fn support_do_bit<DH>(dns_handle: &mut DH) -> impl Future<Item=TestResult, Error=DH::Error>
    where DH: DnsHandle
{
    // Create a query
    let query = Query::query(TESTING_SERVER.clone(), RecordType::A);
    // Create an EDNS struct
    let mut edns = Edns::new();
    edns.set_dnssec_ok(true);
    // Finally, assemble a message
    let mut msg = Message::new();
    msg.add_query(query);
    msg.set_edns(edns);
    dns_handle
        .send(msg)
        .map(|msg| {
            if let Some(edns) = msg.edns() {
                if edns.dnssec_ok() {
                    TestResult::Success
                } else {
                    TestResult::Fail("DO not set")
                }
            } else {
                TestResult::Fail("No EDNS option")
            }
        })
}

fn run_tests(address: std::net::SocketAddr) -> Result<(), Error> {
    // create connections
    let udp_conn = UdpClientConnection::new(address).unwrap();
    let tcp_conn = TcpClientConnection::new(address).unwrap();

    // instantiate tokio.rs reactor
    let mut reactor = Core::new().unwrap();
    let handle = &reactor.handle();

    // UDP stream, where stream is a series of Futures??
    let (udp_stream, udp_stream_handle) = udp_conn.new_stream(handle).unwrap();
    let (tcp_stream, tcp_stream_handle) = tcp_conn.new_stream(handle).unwrap();

    // run basic UDP test
    let mut udp_client_handle = ClientFuture::new(udp_stream, udp_stream_handle, handle, None);
    println!("[{}] Basic UDP: {:?}", address, reactor.run(support_simple_answers(&mut udp_client_handle)));

    // run basic TCP test
    let mut tcp_client_handle = ClientFuture::new(tcp_stream, tcp_stream_handle, handle, None);
    println!("[{}] Basic TCP: {:?}", address, reactor.run(support_simple_answers(&mut tcp_client_handle)));

    // run edns0 test
    println!("[{}] Edns0 UDP: {:?}", address, reactor.run(support_edns0(&mut udp_client_handle)));
    println!("[{}] DO UDP: {:?}", address, reactor.run(support_do_bit(&mut udp_client_handle)));
    Ok(())
}

fn main() {
    let address = "127.0.0.1:53".parse().unwrap();
    //let address = "8.8.8.8:53".parse().unwrap();

    if let Some(_) = env::args().nth(1) {
        println!("With args...");
    } else {
        // no arg
        let _ = run_tests(address);
        let address = "8.8.8.8:53".parse().unwrap();
        let _ = run_tests(address);
        let address = "1.1.1.1:53".parse().unwrap();
        let _ = run_tests(address);
    }
}
