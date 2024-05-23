use clap::Parser;
use tracing_subscriber::prelude::*;

use std::{
    io::Cursor,
    net::{SocketAddr, UdpSocket},
};

use dirt::{
    message::{Message, MsgSection, Result as MsgResult},
    qtype::QType,
};

#[derive(Parser)]
#[command(author, version, about)]
struct Arguments {
    /// Requested domain name
    request: String,
    /// Requested IP V6 domains
    #[arg(short = '6', long = "ipv6", default_value_t)]
    ip_v6: bool,
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);

    let filter_layer = tracing_subscriber::EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();

    let args = Arguments::parse();

    match lookup_domain(&args) {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}

/// Returns a ready-to-use UDP socket connected to the given address
#[tracing::instrument]
fn setup_udp_socket_to(dns_server_addr: SocketAddr) -> std::io::Result<UdpSocket> {
    let udp_sock = match dns_server_addr {
        SocketAddr::V4(_) => UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?,
        SocketAddr::V6(_) => UdpSocket::bind((std::net::Ipv6Addr::UNSPECIFIED, 0))?,
    };

    tracing::trace!(
        "Successfully bound to local addr: {}",
        udp_sock.local_addr().unwrap()
    );

    udp_sock.connect(dns_server_addr)?;
    tracing::trace!("Successfully connected to remote addr: {dns_server_addr}");

    Ok(udp_sock)
}

#[tracing::instrument(fields(question = query.get_query().qname.to_string()), skip(query))]
fn send_query(query: Message, server_addr: std::net::IpAddr) -> MsgResult<Message> {
    let socket_addr = std::net::SocketAddr::from((server_addr, 53));

    // connection setup
    let udp_sock = setup_udp_socket_to(socket_addr)?;

    // query request
    tracing::trace!("Sending query for {:?} via {udp_sock:?}", query.get_query());
    udp_sock.send(&query.query_into_bytes())?;
    tracing::trace!("Query sent successfully");

    // get response
    let mut recv_buf = [0u8; 1024];
    tracing::trace!("Beginning to parse response");
    let bytes_recv = udp_sock.recv(&mut recv_buf)?;
    tracing::trace!("Received {bytes_recv} bytes in response");

    // parse response to message
    let mut msg_bytes_reader = Cursor::new(&recv_buf[..bytes_recv]);

    Message::from_bytes(&mut msg_bytes_reader)
}

fn lookup_domain(args: &Arguments) -> MsgResult<std::net::IpAddr> {
    match args.ip_v6 {
        true => resolve(&args.request, QType::AAAA),
        false => resolve(&args.request, QType::A),
    }
}

pub fn resolve(domain_name: &str, record_type: QType) -> MsgResult<std::net::IpAddr> {
    let mut nameserver = match record_type {
        QType::AAAA => std::net::IpAddr::V6("2001:503:ba3e::2:30".parse().unwrap()),
        _ => std::net::IpAddr::V4(std::net::Ipv4Addr::new(198, 41, 0, 4)),
    };
    loop {
        tracing::info!("Querying {nameserver} for \"{domain_name}\"");
        let query = Message::new_query(domain_name, record_type, false, false);

        let resp = send_query(query, nameserver)?;

        tracing::debug!("Received response: {:?}\n{:?}", resp.header, resp);

        if let Some(domain_ip_rr) = resp.get_record_by_type_from(record_type, MsgSection::Answers) {
            tracing::debug!(
                "Found answer for \"{domain_name}\": {}",
                domain_ip_rr.data_as_ip_addr()
            );
            return Ok(domain_ip_rr.data_as_ip_addr());
        } else if let Some(ns_ip_rr) =
            resp.get_record_by_type_from(record_type, MsgSection::Additionals)
        {
            nameserver = ns_ip_rr.data_as_ip_addr();
            tracing::debug!("Referred to new nameserver: {nameserver}");
        } else if let Some(ns_dname_rr) =
            resp.get_record_by_type_from(QType::NS, MsgSection::Authorities)
        {
            tracing::debug!(
                "Found name for new nameserver: \"{}\"",
                ns_dname_rr.data_as_str()
            );
            nameserver = resolve(ns_dname_rr.data_as_str(), record_type)?;
            tracing::debug!(
                "Resolved new namserver \"{}\": {nameserver}",
                ns_dname_rr.data_as_str()
            );
        } else if let Some(cname_rr) =
            resp.get_record_by_type_from(QType::CNAME, MsgSection::Answers)
        {
            tracing::debug!(
                "Found alias \"{}\" for \"{domain_name}\"",
                cname_rr.data_as_str()
            );
            return resolve(cname_rr.data_as_str(), record_type);
        } else {
            panic!("Unexpected resolver error\nreceived: {resp:#?}")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_build_query() -> std::fmt::Result {
        let correct_bytes_str =
            "82980100000100000000000003777777076578616d706c6503636f6d0000010001";
        let query = Message::new_query("www.example.com", dirt::qtype::QType::A, false, true);
        let query_bytes = query.query_into_bytes();

        let mut query_bytes_str = String::with_capacity(correct_bytes_str.len());

        use std::fmt::Write;
        for byte in query_bytes {
            write!(&mut query_bytes_str, "{byte:02x}")?;
        }

        // Skip first 2 bytes (random id)
        // 2 chars per byte as formatted above --> 4 chars to skip
        assert_eq!(
            query_bytes_str[4..],
            correct_bytes_str[4..],
            "Byte value mismatch"
        );

        Ok(())
    }

    #[test]
    fn test_send_query() -> std::io::Result<()> {
        let query = Message::new_query("www.example.com", dirt::qtype::QType::A, false, true);
        let query_bytes = query.query_into_bytes();

        // connection setup
        let udp_sock =
            setup_udp_socket_to("8.8.8.8:53".parse().unwrap()).expect("Failed to setup UDP socket");

        // query request
        udp_sock.send(&query_bytes).expect("Couldn't send query");

        Ok(())
    }

    #[test]
    fn test_resolve() -> MsgResult<()> {
        let result_ip = resolve("www.example.com", QType::A).expect("Failed to resolve");
        let correct_ip = "93.184.216.34".parse::<std::net::Ipv4Addr>().unwrap();
        assert_eq!(result_ip, correct_ip);
        Ok(())
    }

    #[test]
    fn test_cname() -> MsgResult<()> {
        // facebook has multiple IP addrs, no sense checking for any possible one.
        let _ = resolve("www.facebook.com", QType::A).expect("Failed to lookup domain");
        Ok(())
    }

    #[test]
    #[ignore = "GitHub CI doesn't support IPv6.\nSee https://github.com/actions/runner-images/issues/668"]
    fn test_resolve_ipv6() -> MsgResult<()> {
        let result_ip = resolve("www.example.com", QType::AAAA).expect("Failed to resolve");
        let correct_ip = "2606:2800:220:1:248:1893:25c8:1946"
            .parse::<std::net::Ipv6Addr>()
            .unwrap();
        assert_eq!(result_ip, correct_ip);
        Ok(())
    }
}
