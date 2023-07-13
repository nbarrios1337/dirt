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

use crate::{dname::DomainName, qclass::QClass, qtype::QType};

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
    /// Converts a [`Question`] to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        let mut buf = self.qname.into_bytes();

        buf.write_u16::<NetworkEndian>(self.qtype.into()).unwrap();
        buf.write_u16::<NetworkEndian>(self.qclass.into()).unwrap();

        buf
    }

    /// Reads a [`Question`] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let qname = DomainName::from_bytes(bytes)?;
        let qtype = QType::try_from(bytes.read_u16::<NetworkEndian>()?)?;
        let qclass = QClass::try_from(bytes.read_u16::<NetworkEndian>()?)?;

        Ok(Self {
            qname,
            qtype,
            qclass,
        })
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Wraps the errors that may be encountered during byte decoding of a [`Question`]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse question data: {0}")]
    Io(#[from] std::io::Error),
    /// Stores an error encountered while parsing the [DomainName]
    #[error(transparent)]
    Name(#[from] crate::dname::Error),
    /// Stores an error encountered while parsin the [QType]
    #[error("Failed to convert primitive to QType: {0}")]
    Type(#[from] num_enum::TryFromPrimitiveError<QType>),
    /// Stores an error encountered while parsin the [QClass]
    #[error("Failed to convert primitive to QClass: {0}")]
    Class(#[from] num_enum::TryFromPrimitiveError<QClass>),
}

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
        let result_bytes = question.into_bytes();

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
