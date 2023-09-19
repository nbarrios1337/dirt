//! Domain names in messages are expressed in terms of a sequence of labels.
//!
//! See more in [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
//! and [RFC 1035 section 3.1](https://datatracker.ietf.org/doc/html/rfc1035#section-3.1)

pub(crate) mod label {

    use thiserror::Error;

    /// Labels are the individual nodes or components of a [`DomainName`]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct Label(pub(crate) String);

    impl Label {
        /// The maximum size of a single label within a domain name
        pub const MAX_LABEL_SIZE: usize = 63;

        pub fn new(string: String) -> Self {
            Self(string)
        }
    }

    /// Wraps the errors that may be encountered during byte decoding of a [`Label`]
    #[derive(Debug, Error)]
    pub enum Error {
        /// Stores an error encountered while using [std::io] traits and structs
        #[error("Failed to read {dest_amt} bytes from {src_amt} byte buffer:\n\t{source}")]
        Io {
            src_amt: usize,
            dest_amt: usize,
            source: std::io::Error,
        },
        /// Stores an error encountered while converting from a sequence of [u8] to [String]
        #[error("Failed to convert byte slice {bytes:?} to string slice:\n\t{source}")]
        Convert {
            bytes: Vec<u8>,
            source: std::str::Utf8Error,
        },
    }

    pub(crate) type Result<T> = std::result::Result<T, Error>;
}

pub(crate) use label::Label;

use thiserror::Error;

/// Domain names define a name of a node in requests and responses
#[derive(Clone, PartialEq, Eq)]
pub struct DomainName(pub(crate) Vec<Label>);

impl DomainName {
    /// The maximum number of octets that represent a domain name (i.e., the sum of all label octets and label lengths)
    pub const MAX_NAME_SIZE: usize = 255;
    pub const MAX_UDP_MSG_SIZE: usize = 512;
    /// a domain name is terminated by a length byte of zero
    pub const TERMINATOR: u8 = 0;
}

impl std::fmt::Debug for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DomainName")
            .field(&String::from(self.clone()))
            .finish()
    }
}

impl DomainName {
    pub const fn is_compressed(size: u8) -> bool {
        size & 0b1100_0000 == 0b1100_0000
    }

    /// Creates a new [`DomainName`]
    pub fn new(domain_name: &str) -> Self {
        Self::from(domain_name.to_string())
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Wraps the errors that may be encountered during byte decoding of a [`DomainName`]
#[derive(Debug, Error)]
pub enum Error {
    /// Stores an error encountered while [Label] parsing
    #[error("Attempted to parse a {size}-octets label:\n\t{source}")]
    Label {
        size: u8,
        #[source]
        source: label::Error,
    },
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse domain name data:\n\t{0}")]
    Io(#[from] std::io::Error),
}
