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

use std::io::Cursor;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

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
        let mut buf = self.qname.to_bytes();

        buf.write_u16::<NetworkEndian>(self.qtype.into()).unwrap();
        buf.write_u16::<NetworkEndian>(self.qclass.into()).unwrap();

        buf
    }

    /// Reads a [Question] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        // domain name parsing
        let qname = DomainName::from_bytes(bytes).map_err(QuestionError::Name)?;

        let qtype = QType::try_from(
            bytes
                .read_u16::<NetworkEndian>()
                .map_err(QuestionError::Io)?,
        )
        .map_err(QuestionError::Type)?;

        let qclass = QClass::try_from(
            bytes
                .read_u16::<NetworkEndian>()
                .map_err(QuestionError::Io)?,
        )
        .map_err(QuestionError::Class)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_question() {
        let question = Question {
            qname: DomainName::new("google.com"),
            qclass: QClass::IN,
            qtype: QType::A,
        };

        let correct_bytes = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let result_bytes = question.to_bytes();

        assert_eq!(result_bytes, correct_bytes);
    }

    #[test]
    fn decode_question() -> Result<()> {
        let correct_question = Question {
            qname: DomainName::new("google.com"),
            qclass: QClass::IN,
            qtype: QType::A,
        };

        let mut bytes = Cursor::new(&b"\x06google\x03com\x00\x00\x01\x00\x01"[..]);
        let result_question = Question::from_bytes(&mut bytes)?;

        assert_eq!(result_question.qname, correct_question.qname);
        assert_eq!(result_question.qtype, correct_question.qtype);
        assert_eq!(result_question.qclass, correct_question.qclass);
        Ok(())
    }
}
