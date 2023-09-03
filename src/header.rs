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

use std::io::Cursor;

use byteorder::{ByteOrder, NetworkEndian, ReadBytesExt};

use thiserror::Error;

/// A four bit field that specifies kind of query in this message.
///
/// This value is set by the originator of a query and copied into the response.
#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
enum OpCode {
    /// A standard query
    Query,
    /// An inverse query
    InverseQuery,
    /// A server status request
    Status,
    /// Reserved for future use
    Reserved,
}

impl Default for OpCode {
    fn default() -> Self {
        Self::Query
    }
}

impl TryFrom<u8> for OpCode {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::InverseQuery),
            2 => Ok(Self::Status),
            3..=15 => Ok(Self::Reserved),
            invalid => Err(format!("Invalid OpCode value: {invalid}")),
        }
    }
}

/// This 4 bit field is set as part of responses.
#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
enum ResponseCode {
    /// No error condition
    NoError,
    /// Format error - The name server was unable to interpret the query.
    FormErr,
    /// Server failure - The name server was unable to process this query
    /// due to a problem with the name server.
    ServFail,
    /// Name Error - Meaningful only for responses from an authoritative name server,
    /// this code signifies that the domain name referenced in the query does not exist.
    NxDomain,
    /// Not Implemented - The name server does not support the requested kind of query.
    NotImp,
    /// Refused - The name server refuses to perform the specified operation for policy reasons.
    ///
    /// For example, a name server may not wish to provide the information
    /// to the particular requester, or a name server may not wish to perform
    /// a particular operation (e.g., zone transfer) for particular data.
    Refused,
    /// Reserved for future use. (6-15)
    Reserved,
}

impl Default for ResponseCode {
    fn default() -> Self {
        Self::NoError
    }
}

impl TryFrom<u8> for ResponseCode {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoError),
            1 => Ok(Self::FormErr),
            2 => Ok(Self::ServFail),
            3 => Ok(Self::NxDomain),
            4 => Ok(Self::NotImp),
            5 => Ok(Self::Refused),
            6..=15 => Ok(Self::Reserved),
            invalid => Err(format!("Invalid ResponseCode value: {invalid}")),
        }
    }
}

/// The set of non-u16 data components of a header
///
/// ```text
///                                1  1  1  1  1  1
///  0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// |QR|   OpCode  |AA|TC|RD|RA|   Z    |  RespCode |
/// +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeaderFlags {
    ///A one bit field that specifies whether this message is a query (0), or a response (1).
    query_response: bool,
    /// see [OpCode]'s docs for more details
    op_code: OpCode,
    /// Authoritative Answer - this bit is valid in responses, and specifies that
    /// the responding name serveris an authority for the domain name in question section.
    ///
    /// Note that the contents of the answer section may have multiple owner names because of aliases.
    /// The AA bit corresponds to the name which matches the query name,
    /// or the first owner name in the answer section.
    auth_answer: bool,
    /// TrunCation - specifies that this message was truncated due to length
    /// greater than that permitted on the transmission channel.
    truncated: bool,
    /// Recursion Desired - this bit may be set in a query and is copied into the response.
    ///
    /// If RD is set, it directs the name server to pursue the query recursively.
    /// Recursive query support is optional.
    recursion_desired: bool,
    /// Recursion Available - this be is set or cleared in a response,
    /// and denotes whether recursive query support is available in the name server.
    recursion_avail: bool,
    /// see [ResponseCode]'s docs for more details
    response_code: ResponseCode,
}

impl HeaderFlags {
    pub fn as_u16(&self) -> u16 {
        // first u8
        let higher: u8 = (self.query_response as u8) << 7
            | u8::from(self.op_code) << 3
            | (self.auth_answer as u8) << 2
            | (self.truncated as u8) << 1
            | self.recursion_desired as u8;

        let lower: u8 = (self.recursion_avail as u8) << 7 | u8::from(self.response_code);

        debug_assert_eq!(
            self,
            &Self::from_u16(u16::from_be_bytes([higher, lower])).unwrap()
        );

        u16::from_be_bytes([higher, lower])
    }

    pub fn from_u16(bytes: u16) -> std::result::Result<Self, String> {
        let [higher, lower] = bytes.to_be_bytes();

        let query_response = (higher >> 7) & 1 != 0;
        let op_code = OpCode::try_from((higher & 0b0111_1000) >> 3)?;
        let auth_answer = (higher >> 2) & 1 != 0;
        let truncated = (higher >> 1) & 1 != 0;
        let recursion_desired = higher & 1 != 0;

        let recursion_avail = (lower >> 7) & 1 != 0;
        let response_code = ResponseCode::try_from(lower & 0b0111_1111)?;

        Ok(Self {
            query_response,
            op_code,
            auth_answer,
            truncated,
            recursion_desired,
            recursion_avail,
            response_code,
        })
    }
}

impl From<HeaderFlags> for u16 {
    fn from(value: HeaderFlags) -> Self {
        value.as_u16()
    }
}

impl TryFrom<u16> for HeaderFlags {
    type Error = String;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        Self::from_u16(value)
    }
}

/// The header includes fields that specify which of the remaining sections are present,
/// and also specifywhether the message is a query or a response, a standard query or some other opcode, etc.
#[derive(Debug, Clone, Copy)] // TODO what other derives needed?
pub struct Header {
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

impl Header {
    /// Convert a header to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        // 6 fields, 2 bytes each
        let mut buf: Vec<u8> = vec![0u8; 6 * std::mem::size_of::<u16>()];
        NetworkEndian::write_u16_into(
            &[
                self.id,
                self.flags,
                self.num_questions,
                self.num_answers,
                self.num_authorities,
                self.num_additionals,
            ],
            &mut buf,
        );
        buf
    }

    /// Reads a header from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let mut buf = [0u16; 6];
        bytes.read_u16_into::<NetworkEndian>(&mut buf)?;
        let [id, flags, num_questions, num_answers, num_authorities, num_additionals]: [u16; 6] =
            buf;

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

/// Wraps the errors that may be encountered during byte decoding of a [`Header`]
#[derive(Debug, Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse header data: {0}")]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_header() {
        let header = Header {
            id: 0x1314,
            flags: 0,
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        };

        let header_bytes = header.into_bytes();

        let correct_bytes: Vec<u8> =
            vec![0x13, 0x14, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

        assert_eq!(header_bytes, correct_bytes);
    }

    #[test]
    fn decode_header() -> Result<()> {
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

        let result_header = Header::from_bytes(&mut Cursor::new(&test_bytes))?;

        assert_eq!(result_header.id, expected_id);
        assert_eq!(result_header.flags, expected_flags);
        assert_eq!(result_header.num_questions, 1);
        assert_eq!(result_header.num_answers, 0);
        assert_eq!(result_header.num_authorities, 0);
        assert_eq!(result_header.num_additionals, 0);

        Ok(())
    }
}
