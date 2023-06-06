//! The question section is used to carry the "question" in most queries, i.e., the parameters that define what is being asked.
//!
//! The section contains QDCOUNT (usually 1) entries, each of the following format:
//!
//! ```text
//!                                     1  1  1  1  1  1
//!       0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                                               |
//!     /                     QNAME                     /
//!     /                                               /
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                     QTYPE                     |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!     |                     QCLASS                    |
//!     +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
//!
//! where:
//!
//! QNAME           a domain name represented as a sequence of labels, where
//!                 each label consists of a length octet followed by that
//!                 number of octets. The domain name terminates with the
//!                 zero length octet for the null label of the root.  Note
//!                 that this field may be an odd number of octets; no
//!                 padding is used.
//!
//! QTYPE           a two octet code which specifies the type of the query.
//!                 The values for this field include all codes valid for a
//!                 TYPE field, together with some more general codes which
//!                 can match more than one type of RR.
//!
//! QCLASS          a two octet code that specifies the class of the query.
//!                 For example, the QCLASS field is IN for the Internet.
//! ```
//!

use crate::{
    dname::{DomainName, DomainNameError},
    qclass::QClass,
    qtype::QType,
};

/// Carries the parameters that define what is being asked
#[derive(Debug, Clone)]
pub struct Question {
    /// a domain name represented as a sequence of labels,
    /// where each label consists of a length octet followed by that number of octets.
    ///
    /// The domain name terminates with the zero length octet for the null label of the root.
    ///
    /// Note that this field may be an odd number of octets; no padding is used
    pub qname: DomainName,
    /// a two octet code which specifies the type of the query.
    ///
    /// The values for this field include all codes valid for a TYPE field,
    /// together with some more general codes which can match more than one type of RR.
    pub qtype: QType,
    /// a two octet code that specifies the class of the query.
    ///
    /// For example, the QCLASS field is IN for the Internet.
    pub qclass: QClass,
}

impl Question {
    /// Converts a [Question] to owned bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.qname.encode_dns_name();

        let qtype_u16: u16 = self.qtype.into();
        buf.extend_from_slice(&qtype_u16.to_be_bytes());

        let qclass_u16: u16 = self.qclass.into();
        buf.extend_from_slice(&qclass_u16.to_be_bytes());

        buf
    }

    /// Reads a [Question] from a slice of bytes
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        use std::io::{BufRead, Read};

        // set up owned buffer for domain name parsing
        let mut question_bytes = Vec::with_capacity(DomainName::MAX_NAME_SIZE);

        // TODO use for question size?
        // Read all the name-related bytes (delimited by zero octet)
        let question_size = bytes
            .read_until(DomainName::TERMINATOR, &mut question_bytes)
            .map_err(QuestionError::Io)?;
        let qname = DomainName::decode_dns_name(&question_bytes).map_err(QuestionError::Name)?;

        // reusable buffer for u16 parsing
        let mut u16_buffer = [0u8; 2];

        // qtype parsing
        bytes
            .read_exact(&mut u16_buffer)
            .map_err(QuestionError::Io)?;
        let qtype = QType::try_from(u16::from_be_bytes(u16_buffer)).map_err(QuestionError::Type)?;

        // qclass parsing
        bytes
            .read_exact(&mut u16_buffer)
            .map_err(QuestionError::Io)?;
        let qclass =
            QClass::try_from(u16::from_be_bytes(u16_buffer)).map_err(QuestionError::Class)?;

        Ok(Self {
            qname,
            qtype,
            qclass,
        })
    }
}

type Result<T> = std::result::Result<T, QuestionError>;

/// [QuestionError] wraps the errors that may be encountered during byte decoding of a [Question]
#[derive(Debug)]
pub enum QuestionError {
    /// Stores an error encountered while using [std::io] traits and structs
    Io(std::io::Error),
    /// Stores an error encountered while parsing the [DomainName]
    Name(DomainNameError),
    /// Stores an error encountered while parsin the [QType]
    Type(num_enum::TryFromPrimitiveError<QType>),
    /// Stores an error encountered while parsin the [QClass]
    Class(num_enum::TryFromPrimitiveError<QClass>),
}

impl std::fmt::Display for QuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Name(e) => write!(f, "Name parsing error: {e}"),
            Self::Type(e) => write!(f, "type parsing error: {e}"),
            Self::Class(e) => write!(f, "class parsing error: {e}"),
        }
    }
}

impl std::error::Error for QuestionError {}
