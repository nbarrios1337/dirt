//! The header includes fields that
//! specify which of the remaining sections are present, and also specify
//! whether the message is a query or a response, a standard query or some
//! other opcode, etc.
//!
//! The header section is always present.
//!
//! The header contains the following fields:
//!
//! ```text
//!                                1  1  1  1  1  1
//!  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                      ID                       |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    QDCOUNT                    |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    ANCOUNT                    |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    NSCOUNT                    |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//! |                    ARCOUNT                    |
//! +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!
//! where:
//! ID              A 16 bit identifier assigned by the program that
//!                 generates any kind of query.  This identifier is copied
//!                 the corresponding reply and can be used by the requester
//!                 to match up replies to outstanding queries.
//! QR              A one bit field that specifies whether this message is a
//!                 query (0), or a response (1).
//! OPCODE          A four bit field that specifies kind of query in this
//!                 message.  This value is set by the originator of a query
//!                 and copied into the response.  The values are:
//!                 0               a standard query (QUERY)
//!                 1               an inverse query (IQUERY)
//!                 2               a server status request (STATUS)
//!                 3-15            reserved for future use
//! AA              Authoritative Answer - this bit is valid in responses,
//!                 and specifies that the responding name server is an
//!                 authority for the domain name in question section.
//!                 Note that the contents of the answer section may have
//!                 multiple owner names because of aliases.  The AA bit
//!                 corresponds to the name which matches the query name, or
//!                 the first owner name in the answer section.
//! TC              TrunCation - specifies that this message was truncated
//!                 due to length greater than that permitted on the
//!                 transmission channel.
//! RD              Recursion Desired - this bit may be set in a query and
//!                 is copied into the response.  If RD is set, it directs
//!                 the name server to pursue the query recursively.
//!                 Recursive query support is optional.
//! RA              Recursion Available - this be is set or cleared in a
//!                 response, and denotes whether recursive query support is
//!                 available in the name server.
//! Z               Reserved for future use.  Must be zero in all queries
//!                 and responses.
//! RCODE           Response code - this 4 bit field is set as part of
//!                 responses.  The values have the following
//!                 interpretation:
//!                 0               No error condition
//!                 1               Format error - The name server was
//!                                 unable to interpret the query.
//!                 2               Server failure - The name server was
//!                                 unable to process this query due to a
//!                                 problem with the name server.
//!                 3               Name Error - Meaningful only for
//!                                 responses from an authoritative name
//!                                 server, this code signifies that the
//!                                 domain name referenced in the query does
//!                                 not exist.
//!                 4               Not Implemented - The name server does
//!                                 not support the requested kind of query.
//!                 5               Refused - The name server refuses to
//!                                 perform the specified operation for
//!                                 policy reasons.  For example, a name
//!                                 server may not wish to provide the
//!                                 information to the particular requester,
//!                                 or a name server may not wish to perform
//!                                 a particular operation (e.g., zone
//!                                 transfer) for particular data.
//!                 6-15            Reserved for future use.
//! QDCOUNT         an unsigned 16 bit integer specifying the number of
//!                 entries in the question section.
//! ANCOUNT         an unsigned 16 bit integer specifying the number of
//!                 resource records in the answer section.
//! NSCOUNT         an unsigned 16 bit integer specifying the number of name
//!                 server resource records in the authority records
//!                 section.
//! ARCOUNT         an unsigned 16 bit integer specifying the number of
//!                 resource records in the additional records section.
//! ```
//!
//! See more in [RFC 1035 section 4.1.1](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1)

use std::io::Read;

/// The header includes fields that specify which of the remaining sections are present,
/// and also specifywhether the message is a query or a response, a standard query or some other opcode, etc.
#[derive(Debug, Clone, Copy)] // TODO what other derives needed?
pub struct DnsHeader {
    /// A 16 bit identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the requester to match up replies to outstanding queries.
    pub id: u16,
    pub flags: u16, // TODO bitflags?
    /// An unsigned 16 bit integer specifying the number of entries in the question section.
    pub num_questions: u16,
    /// An unsigned 16 bit integer specifying the number of resource records in the answer section.
    pub num_answers: u16,
    /// An unsigned 16 bit integer specifying the number of name server resource records in the authority records section.
    pub num_authorities: u16,
    /// An unsigned 16 bit integer specifying the number of resource records in the additional records section.
    pub num_additionals: u16,
}

impl DnsHeader {
    /// Convert a header to owned bytes
    pub fn to_bytes(self) -> Vec<u8> {
        // 6 fields, 2 bytes each
        let mut buf: Vec<u8> = Vec::with_capacity(6 * 2);
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&self.num_questions.to_be_bytes());
        buf.extend_from_slice(&self.num_answers.to_be_bytes());
        buf.extend_from_slice(&self.num_authorities.to_be_bytes());
        buf.extend_from_slice(&self.num_additionals.to_be_bytes());
        buf
    }

    /// Reads a header from a slice of bytes
    pub fn from_bytes(bytes: &mut &[u8]) -> std::io::Result<Self> {
        // reusable buffer for u16 parsing
        let mut u16_buffer = [0u8; 2];

        // id parsing
        bytes.read_exact(&mut u16_buffer)?;
        let id = u16::from_be_bytes(u16_buffer);

        // flags parsing
        bytes.read_exact(&mut u16_buffer)?;
        let flags = u16::from_be_bytes(u16_buffer);

        // num_questions parsing
        bytes.read_exact(&mut u16_buffer)?;
        let num_questions = u16::from_be_bytes(u16_buffer);

        // num_answers parsing
        bytes.read_exact(&mut u16_buffer)?;
        let num_answers = u16::from_be_bytes(u16_buffer);

        // num_authorities parsing
        bytes.read_exact(&mut u16_buffer)?;
        let num_authorities = u16::from_be_bytes(u16_buffer);

        // num_additionals parsing
        bytes.read_exact(&mut u16_buffer)?;
        let num_additionals = u16::from_be_bytes(u16_buffer);

        Ok(Self {
            id,
            flags,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DnsHeader;

    #[test]
    fn encode_header() {
        let header = DnsHeader {
            id: 0x1314,
            flags: 0,
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        };

        let header_bytes = header.to_bytes();

        let correct_bytes: Vec<u8> =
            vec![0x13, 0x14, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

        assert_eq!(header_bytes, correct_bytes);
    }

    #[test]
    fn decode_header() -> std::io::Result<()> {
        let test_bytes = vec![
            0x82, 0x98, // id
            0x01, 0x00, // flags
            0x00, 0x01, // n_q
            0x00, 0x00, // n_ans
            0x00, 0x00, // n_auth
            0x00, 0x00, // n_add
        ];

        let expected_id = 0x8298;

        // recursion desired
        let expected_flags: u16 = 1 << 8;

        let result_header = DnsHeader::from_bytes(&mut &test_bytes[..])?;

        assert_eq!(result_header.id, expected_id);
        assert_eq!(result_header.flags, expected_flags);
        assert_eq!(result_header.num_questions, 1);
        assert_eq!(result_header.num_answers, 0);
        assert_eq!(result_header.num_authorities, 0);
        assert_eq!(result_header.num_additionals, 0);

        Ok(())
    }
}
