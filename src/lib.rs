mod dname;
mod header;
mod message;
mod qclass;
mod qtype;
mod query;
mod question;
mod record;

use std::{
    io::Cursor,
    net::{SocketAddr, UdpSocket},
};

use query::Query;

use crate::{
    message::{Message, MsgSection},
    qtype::QType,
};

/// Returns a ready-to-use UDP socket connected to the given address
fn setup_udp_socket_to(dns_server_addr: SocketAddr) -> std::io::Result<UdpSocket> {
    let udp_sock = match dns_server_addr {
        SocketAddr::V4(_) => UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?,
        SocketAddr::V6(_) => UdpSocket::bind((std::net::Ipv6Addr::UNSPECIFIED, 0))?,
    };
    udp_sock.connect(dns_server_addr)?;
    Ok(udp_sock)
}

fn send_query(query: Query, server_addr: std::net::IpAddr) -> message::Result<Message> {
    let socket_addr = std::net::SocketAddr::from((server_addr, 53));

    // connection setup
    let udp_sock = setup_udp_socket_to(socket_addr)?;

    // query request
    udp_sock.send(&query.into_bytes())?;

    // get response
    let mut recv_buf = [0u8; 1024];
    let bytes_recv = udp_sock.recv(&mut recv_buf)?;

    // parse response to message
    let mut msg_bytes_reader = Cursor::new(&recv_buf[..bytes_recv]);

    Message::from_bytes(&mut msg_bytes_reader)
}

pub fn lookup_domain(domain_name: &str) -> message::Result<std::net::IpAddr> {
    resolve(domain_name, QType::A)
}

pub fn resolve(domain_name: &str, record_type: QType) -> message::Result<std::net::IpAddr> {
    let mut nameserver = std::net::IpAddr::V4(std::net::Ipv4Addr::new(198, 41, 0, 4));
    loop {
        tracing::info!("Querying {nameserver} for \"{domain_name}\"");
        let query = Query::new(domain_name, record_type, 0);
        tracing::debug!("Sending query: {query:?}");
        let resp = send_query(query, nameserver)?;

        tracing::debug!("Received response: {:?}", resp.header);

        if let Some(domain_ip_rr) = resp.get_record_by_type_from(QType::A, MsgSection::Answers) {
            tracing::debug!(
                "Found answer for \"{domain_name}\": {}",
                domain_ip_rr.data_as_ip_addr()
            );
            return Ok(domain_ip_rr.data_as_ip_addr());
        } else if let Some(ns_ip_rr) =
            resp.get_record_by_type_from(QType::A, MsgSection::Additionals)
        {
            nameserver = ns_ip_rr.data_as_ip_addr();
            tracing::debug!("Referred to new nameserver: {nameserver}")
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
            )
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

    // endianness clarification: 7th MSB of the 3rd octet is 9 bits away from bit 15.
    const RECURSION_DESIRED: u16 = 1 << 8;

    #[test]
    fn test_build_query() -> std::fmt::Result {
        let correct_bytes_str =
            "82980100000100000000000003777777076578616d706c6503636f6d0000010001";
        let query = Query::new("www.example.com", qtype::QType::A, RECURSION_DESIRED);
        let query_bytes = query.into_bytes();

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
        let query = Query::new("www.example.com", qtype::QType::A, RECURSION_DESIRED);
        let query_bytes = query.into_bytes();

        // connection setup
        let udp_sock =
            setup_udp_socket_to("8.8.8.8:53".parse().unwrap()).expect("Failed to setup UDP socket");

        // query request
        udp_sock.send(&query_bytes).expect("Couldn't send query");

        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<(), message::Error> {
        let result_ip = resolve("www.example.com", QType::A)?;
        let correct_ip = "93.184.216.34".parse::<std::net::Ipv4Addr>().unwrap();
        assert_eq!(result_ip, correct_ip);
        Ok(())
    }

    #[test]
    fn test_cname() -> message::Result<()> {
        // facebook has multiple IP addrs, no sense checking for any possible one.
        let _ = lookup_domain("www.facebook.com")?;
        Ok(())
    }
}
