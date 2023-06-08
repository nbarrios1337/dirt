//! Domain names in messages are expressed in terms of a sequence of labels.
//!
//! See more in [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
//! and [RFC 1035 section 3.1](https://datatracker.ietf.org/doc/html/rfc1035#section-3.1)

/// Domain names define a name of a node in requests and responses
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(String);

impl DomainName {
    /// The maximum size of a single label within a domain name
    pub const MAX_LABEL_SIZE: usize = 63;
    /// The maximum number of octets that represent a domain name (i.e., the sum of all label octets and label lengths)
    pub const MAX_NAME_SIZE: usize = 255;
    pub const MAX_UDP_MSG_SIZE: usize = 512;
    /// a domain name is terminated by a length byte of zero
    pub const TERMINATOR: u8 = 0;
}

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName(value)
    }
}

impl DomainName {
    /// Converts a [DomainName] to owned bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut encoded: Vec<u8> = self
            .0
            .split('.')
            .map(|substr| (substr.len(), substr.to_string()))
            .flat_map(|(len, mut substr)| {
                substr.insert(0, len as u8 as char);
                substr.into_bytes()
            })
            .collect();
        encoded.push(0);
        encoded
    }

    /// Reads a [DomainName] from a slice of bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        use std::io::prelude::*;

        // buffers and metadata storage
        let mut bytes_cursor = std::io::Cursor::new(bytes);
        let mut label_bytes_buffer = [0u8; Self::MAX_LABEL_SIZE];
        let mut cur_label_length_slice = [0u8];

        let mut labels: Vec<String> = Vec::new();

        while (bytes_cursor.position() as usize) < bytes.len() {
            // get length
            bytes_cursor
                .read_exact(&mut cur_label_length_slice)
                .map_err(DomainNameError::Io)?;
            let cur_label_length = u8::from_be_bytes(cur_label_length_slice);

            // found the name delimiter
            if cur_label_length == 0 {
                break;
            }

            // set up exact buffer for label read
            let cur_label_bytes = &mut label_bytes_buffer[0..cur_label_length as usize];
            bytes_cursor
                .read_exact(cur_label_bytes)
                .map_err(DomainNameError::Io)?;

            let cur_label = std::str::from_utf8(cur_label_bytes)
                .map_err(DomainNameError::Parse)?
                .to_string();

            labels.push(cur_label);
        }

        Ok(Self(labels.join(".")))
    }

    /// Creates a new [DomainName]
    pub fn new(domain_name: &str) -> Self {
        DomainName(domain_name.to_string())
    }
}

type Result<T> = std::result::Result<T, DomainNameError>;

/// [DomainNameError] wraps the errors that may be encountered during byte decoding of a [DomainName]
#[derive(Debug)]
pub enum DomainNameError {
    /// Stores an error encountered while using [std::io] traits and structs
    Io(std::io::Error),
    /// Stores an error encountered while using [std::convert] traits and structs
    Parse(std::str::Utf8Error),
}

impl std::fmt::Display for DomainNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainNameError::Io(io) => write!(f, "IO error: {io}"),
            DomainNameError::Parse(parse) => write!(f, "String parsing error: {parse}"),
        }
    }
}

impl std::error::Error for DomainNameError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests encoding of "google.com"
    #[test]
    fn encode_dname() {
        let correct_bytes = b"\x06google\x03com\x00";

        let google_domain = DomainName::new("google.com");
        let result_bytes = google_domain.to_bytes();

        assert_eq!(result_bytes, correct_bytes);
    }

    /// Tests decoding of "google.com"
    #[test]
    fn decode_dname() -> Result<()> {
        let correct_dname = DomainName::new("google.com");
        let google_domain_bytes = b"\x06google\x03com\x00";

        let result_dname = DomainName::from_bytes(google_domain_bytes)?;

        assert_eq!(result_dname, correct_dname);

        Ok(())
    }
}
