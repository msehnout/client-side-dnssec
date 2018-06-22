use connections::{ForwardZone, ReverseZone};
use failure::Error;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

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

pub fn apply_rules(fwd_zones: Vec<ForwardZone>, reverse_zones: Vec<ReverseZone>) -> Result<(), Error> {
    let mut stream = UnixStream::connect("/run/knot-resolver/control@1")?;
    for i in &fwd_zones {
        let policy_rule = format!("policy.add(policy.suffix(policy.STUB('{}'), {{todname('{}')}}))\n", i.nameservers[0], i.domain);
        print!("{}", policy_rule);
        stream.write_all(policy_rule.as_bytes())?;
        let mut reader = BufReader::new(&stream);
        read_response_from_socket(&mut reader);
    }
    for i in reverse_zones {
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

// TODO: remove all rules