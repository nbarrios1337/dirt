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

use crate::{dname::DomainName, qclass::QClass, qtype::QType};

/// Carries the parameters that define what is being asked
#[derive(Debug, Clone, PartialEq, Eq)]
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

pub(crate) type Result<T> = std::result::Result<T, Error>;

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
