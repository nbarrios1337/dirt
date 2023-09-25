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

use rand::Rng;
use thiserror::Error;

/// A four bit field that specifies kind of query in this message.
///
/// This value is set by the originator of a query and copied into the response.
#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum OpCode {
    /// A standard query
    #[default]
    Query,
    /// An inverse query
    InverseQuery,
    /// A server status request
    Status,
    /// Reserved for future use
    Reserved,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Query),
            1 => Ok(Self::InverseQuery),
            2 => Ok(Self::Status),
            3..=15 => Ok(Self::Reserved),
            invalid => Err(Error::OpCode(invalid)),
        }
    }
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Query => f.write_str("Query"),
            OpCode::InverseQuery => f.write_str("Inverse Query"),
            OpCode::Status => f.write_str("Status"),
            OpCode::Reserved => f.write_str("Reserved"),
        }
    }
}

/// This 4 bit field is set as part of responses.
#[derive(Debug, Clone, Copy, num_enum::IntoPrimitive, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum ResponseCode {
    /// No error condition
    #[default]
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

impl TryFrom<u8> for ResponseCode {
    type Error = Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NoError),
            1 => Ok(Self::FormErr),
            2 => Ok(Self::ServFail),
            3 => Ok(Self::NxDomain),
            4 => Ok(Self::NotImp),
            5 => Ok(Self::Refused),
            6..=15 => Ok(Self::Reserved),
            invalid => Err(Error::ResponseCode(invalid)),
        }
    }
}

impl std::fmt::Display for ResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseCode::NoError => f.write_str("No Error"),
            ResponseCode::FormErr => f.write_str("Format Error"),
            ResponseCode::ServFail => f.write_str("Server Failure"),
            ResponseCode::NxDomain => f.write_str("Nonexistent Domain"),
            ResponseCode::NotImp => f.write_str("Not Implemented"),
            ResponseCode::Refused => f.write_str("Refused"),
            ResponseCode::Reserved => f.write_str("Reserved"),
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

// non-consuming builders
impl HeaderFlags {
    /// Set the QR bit for this header
    pub fn set_qr(&mut self, qr: bool) -> &mut Self {
        self.query_response = qr;
        self
    }

    /// Request an authoritative answer for this header
    pub fn set_authoritative(&mut self, aa: bool) -> &mut Self {
        self.auth_answer = aa;
        self
    }

    /// Set the truncation bit for this header
    pub fn set_truncated(&mut self, tc: bool) -> &mut Self {
        self.truncated = tc;
        self
    }

    /// Request an authoritative answer for this header
    pub fn set_recursion_desired(&mut self, rd: bool) -> &mut Self {
        self.recursion_desired = rd;
        self
    }

    /// Set the recursion available bit for this header
    pub fn set_recursion_avail(&mut self, ra: bool) -> &mut Self {
        self.recursion_avail = ra;
        self
    }

    /// Sets the [`OpCode`] for this header
    pub fn set_op_code(&mut self, op: u8) -> Result<&mut Self> {
        self.op_code = OpCode::try_from(op)?;
        Ok(self)
    }

    /// Sets the [`ResponseCode`] for this header
    pub fn set_response_code(&mut self, resp: u8) -> Result<&mut Self> {
        self.response_code = ResponseCode::try_from(resp)?;
        Ok(self)
    }

    pub fn finalize(&mut self) -> Self {
        *self
    }
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

    pub fn gen_query_header(
        query_type: u8,
        authoritative: bool,
        recursion_desired: bool,
    ) -> Result<Self> {
        Ok(Self {
            id: rand::thread_rng().gen(),
            flags: HeaderFlags::default()
                .set_op_code(query_type)?
                .set_authoritative(authoritative)
                .set_recursion_desired(recursion_desired)
                .finalize(),
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        })
    }
}

/// Wraps the errors that may be encountered during byte decoding of a [`Header`]
#[derive(Debug, Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse header data: {0}")]
    Io(#[from] std::io::Error),
    /// Stores an error encountered while parsing the [OpCode]
    #[error("Failed to convert primitive to OpCode: {0}")]
    OpCode(u8),
    /// Stores an error encountered while parsing the [ResponseCode]
    #[error("Failed to convert primitive to ResponseCode: {0}")]
    ResponseCode(u8),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
