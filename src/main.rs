mod header;
mod qclass;
mod qtype;
mod question;
mod record;

use rand::Rng;

use header::DnsHeader;
use qtype::QType;
use question::*;

pub fn build_query(domain_name: &str, record_type: u16) -> Vec<u8> {
    let id: u16 = rand::thread_rng().gen();
    // endianness clarification: 7th MSB of the 3rd octet is 9 bits away from bit 15.
    const RECURSION_DESIRED: u16 = 1 << 8;
    let header = DnsHeader {
        id,
        flags: RECURSION_DESIRED,
        num_questions: 1,
        num_answers: 0,
        num_authorities: 0,
        num_additionals: 0,
    };

    let name = DomainName::new(domain_name);
    let question = Question {
        qname: name,
        qclass: qclass::QClass::IN,
        qtype: QType::try_from(record_type).unwrap(),
    };

    let mut header_bytes = header.to_bytes();
    let mut question_bytes = question.to_bytes();
    let mut buf = Vec::with_capacity(header_bytes.len() + question_bytes.len());
    buf.append(&mut header_bytes);
    buf.append(&mut question_bytes);
    buf
}

fn main() -> std::io::Result<()> {
    use std::net::UdpSocket;

    let query_bytes = build_query("www.example.com", qtype::QType::A.into());
    let dns_server_addr = "8.8.8.8:53";

    // connection setup
    let udp_sock = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))
        .unwrap_or_else(|e| panic!("Couldn't bind to local address -- {e}"));

    udp_sock
        .connect(dns_server_addr)
        .unwrap_or_else(|e| panic!("Couldn't connect to DNS Server @ {dns_server_addr} -- {e}"));

    // query request
    udp_sock.send(&query_bytes).expect("Couldn't send query");

    // query response
    let mut recv_buf = [0u8; 1024];
    match udp_sock.recv(&mut recv_buf) {
        Ok(bytes_sent) => {
            let recv_bytes = &recv_buf[..bytes_sent];
            print_bytes_as_hex(recv_bytes);
        }
        Err(e) => {
            let mut query_id_bytes = [0u8; 2];
            query_id_bytes.clone_from_slice(&query_bytes[0..2]);
            let query_id = u16::from_be_bytes(query_id_bytes);
            eprintln!("No response returned for query {query_id} -- {e}");
        }
    }

    Ok(())
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

    /// Tests encoding of "google.com"
    #[test]
    fn qname_encoding() {
        let correct_bytes = b"\x06google\x03com\x00";

        let google_domain = DomainName::new("google.com");
        let result_bytes = google_domain.encode_dns_name();

        assert_eq!(result_bytes, correct_bytes);
    }

    #[test]
    fn test_build_query() -> std::fmt::Result {
        let correct_bytes_str =
            "82980100000100000000000003777777076578616d706c6503636f6d0000010001";
        let query_bytes = build_query("www.example.com", qtype::QType::A.into());

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

    /// Returns a ready-to-use UDP socket connected to Google's DNS server
    fn socket_setup() -> std::net::UdpSocket {
        // Google DNS server address
        let dns_server_addr = "8.8.8.8:53";

        // Bind to any available local address and port
        let udp_sock = std::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))
            .unwrap_or_else(|e| panic!("Couldn't bind to local address -- {e}"));

        udp_sock.connect(dns_server_addr).unwrap_or_else(|e| {
            panic!("Couldn't connect to DNS Server @ {dns_server_addr} -- {e}")
        });

        udp_sock
    }

    #[test]
    fn test_send_query() -> std::io::Result<()> {
        let query_bytes = build_query("www.example.com", qtype::QType::A.into());

        // connection setup
        let udp_sock = socket_setup();

        // query request
        udp_sock.send(&query_bytes).expect("Couldn't send query");

        Ok(())
    }
}
