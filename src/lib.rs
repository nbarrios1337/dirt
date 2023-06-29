mod dname;
mod header;
mod message;
mod qclass;
mod qtype;
mod question;
mod record;

use std::{
    io::Cursor,
    net::{ToSocketAddrs, UdpSocket},
};

use rand::Rng;

use crate::{
    dname::DomainName,
    header::Header,
    message::{Message, MessageError},
    qtype::QType,
    question::Question,
};

pub fn build_query(domain_name: &str, record_type: QType, flags: u16) -> Vec<u8> {
    let id: u16 = rand::thread_rng().gen();
    let header = Header {
        id,
        flags,
        num_questions: 1,
        num_answers: 0,
        num_authorities: 0,
        num_additionals: 0,
    };

    let name = DomainName::new(domain_name);
    let question = Question {
        qname: name,
        qclass: qclass::QClass::IN,
        qtype: record_type,
    };

    let mut header_bytes = header.into_bytes();
    let mut question_bytes = question.into_bytes();
    let mut buf = Vec::with_capacity(header_bytes.len() + question_bytes.len());
    buf.append(&mut header_bytes);
    buf.append(&mut question_bytes);
    buf
}

/// Returns a ready-to-use UDP socket connected to the given address
fn setup_udp_socket_to(dns_server_addr: impl ToSocketAddrs) -> std::io::Result<UdpSocket> {
    let udp_sock = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?;
    udp_sock.connect(dns_server_addr)?;
    Ok(udp_sock)
}

fn send_query(
    desired_addr: &str,
    server_addr: std::net::IpAddr,
    record_type: QType,
) -> Result<Message, MessageError> {
    let query = build_query(desired_addr, record_type, 0);

    let socket_addr = std::net::SocketAddr::from((server_addr, 53));

    // connection setup
    let udp_sock = setup_udp_socket_to(socket_addr)?;

    // query request
    udp_sock.send(&query)?;

    // get response
    let mut recv_buf = [0u8; 1024];
    let bytes_recv = udp_sock.recv(&mut recv_buf)?;

    // parse response to message
    let mut msg_bytes_reader = Cursor::new(&recv_buf[..bytes_recv]);

    Message::from_bytes(&mut msg_bytes_reader)
}

pub fn lookup_domain(domain_name: &str) -> Result<std::net::Ipv4Addr, MessageError> {
    resolve(domain_name, QType::A)
}

pub fn resolve(domain_name: &str, record_type: QType) -> Result<std::net::Ipv4Addr, MessageError> {
    let mut nameserver = "198.41.0.4".parse::<std::net::IpAddr>().unwrap();
    loop {
        println!("Querying {nameserver} for {domain_name}");
        let resp = send_query(domain_name, nameserver, record_type)?;

        // have an answer, return
        if let Some(domain_ip) = resp.answers.iter().find(|answer| answer.qtype == QType::A) {
            return Ok(std::net::Ipv4Addr::from(
                <[u8; 4]>::try_from(&domain_ip.rdata[..4]).unwrap(),
            ));
        } else if let Some(nameserver_ip) =
            resp.additionals.iter().find(|addi| addi.qtype == QType::A)
        {
            nameserver = std::net::IpAddr::V4(std::net::Ipv4Addr::from(
                <[u8; 4]>::try_from(&nameserver_ip.rdata[..4]).unwrap(),
            ))
        } else if let Some(new_nameserver) =
            resp.authorities.iter().find(|auth| auth.qtype == QType::NS)
        {
            nameserver = std::net::IpAddr::V4(resolve(
                std::str::from_utf8(&new_nameserver.rdata).unwrap(),
                record_type,
            )?);
        } else {
            panic!("Unexpected resolver error\nreceived: {resp:#?}")
        }
    }
}

fn print_bytes_as_hex(bytes: &[u8]) {
    eprint!("0x");
    for b in bytes {
        eprint!("{b:02X?}")
    }
    eprintln!();
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
        let query_bytes = build_query("www.example.com", qtype::QType::A, RECURSION_DESIRED);

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
        let query_bytes = build_query("www.example.com", qtype::QType::A, RECURSION_DESIRED);

        // connection setup
        let udp_sock = setup_udp_socket_to("8.8.8.8:53").expect("Failed to setup UDP socket");

        // query request
        udp_sock.send(&query_bytes).expect("Couldn't send query");

        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<(), message::MessageError> {
        let result_ip = resolve("www.example.com", QType::A)?;
        let correct_ip = "93.184.216.34".parse::<std::net::Ipv4Addr>().unwrap();
        assert_eq!(result_ip, correct_ip);
        Ok(())
    }
}
