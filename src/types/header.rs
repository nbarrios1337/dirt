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

use thiserror::Error;

/// A four bit field that specifies kind of query in this message.
///
/// This value is set by the originator of a query and copied into the response.
#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum OpCode {
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
pub(crate) enum ResponseCode {
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
    pub(crate) query_response: bool,
    /// see [OpCode]'s docs for more details
    pub(crate) op_code: OpCode,
    /// Authoritative Answer - this bit is valid in responses, and specifies that
    /// the responding name serveris an authority for the domain name in question section.
    ///
    /// Note that the contents of the answer section may have multiple owner names because of aliases.
    /// The AA bit corresponds to the name which matches the query name,
    /// or the first owner name in the answer section.
    pub(crate) auth_answer: bool,
    /// TrunCation - specifies that this message was truncated due to length
    /// greater than that permitted on the transmission channel.
    pub(crate) truncated: bool,
    /// Recursion Desired - this bit may be set in a query and is copied into the response.
    ///
    /// If RD is set, it directs the name server to pursue the query recursively.
    /// Recursive query support is optional.
    pub(crate) recursion_desired: bool,
    /// Recursion Available - this be is set or cleared in a response,
    /// and denotes whether recursive query support is available in the name server.
    pub(crate) recursion_avail: bool,
    /// see [ResponseCode]'s docs for more details
    pub(crate) response_code: ResponseCode,
}

/// The header includes fields that specify which of the remaining sections are present,
/// and also specifywhether the message is a query or a response, a standard query or some other opcode, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // TODO what other derives needed?
pub struct Header {
    /// A 16 bit identifier assigned by the program that generates any kind of query.
    /// This identifier is copied the corresponding reply and can be used by the requester to match up replies to outstanding queries.
    pub id: u16,
    pub flags: HeaderFlags,
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
    pub fn new(id: u16, flags: HeaderFlags) -> Self {
        Self {
            id,
            flags,
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        }
    }
}

/// Wraps the errors that may be encountered during byte decoding of a [`Header`]
#[derive(Debug, Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse header data: {0}")]
    Io(#[from] std::io::Error),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
