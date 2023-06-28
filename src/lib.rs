mod dname;
mod header;
mod packet;
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
    packet::{Packet, PacketError},
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
    server_addr: impl ToSocketAddrs,
    record_type: QType,
) -> Result<Packet, PacketError> {
    let query = build_query(desired_addr, record_type, 0);

    // connection setup
    let udp_sock = setup_udp_socket_to(server_addr)?;

    // query request
    udp_sock.send(&query)?;

    // get response
    let mut recv_buf = [0u8; 1024];
    let bytes_recv = udp_sock.recv(&mut recv_buf)?;

    // parse response to packet
    let mut pkt_bytes_reader = Cursor::new(&recv_buf[..bytes_recv]);

    Packet::from_bytes(&mut pkt_bytes_reader)
}

pub fn lookup_domain(domain_name: &str) -> Result<std::net::Ipv4Addr, packet::PacketError> {
    send_query(domain_name, "8.8.8.8:53", QType::A).map(|pkt| {
        std::net::Ipv4Addr::from(<[u8; 4]>::try_from(&pkt.answers[0].rdata[..4]).unwrap())
    })
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
}
