//! Domain names in messages are expressed in terms of a sequence of labels.
//!
//! See more in [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
//! and [RFC 1035 section 3.1](https://datatracker.ietf.org/doc/html/rfc1035#section-3.1)

use std::io::{Cursor, Read};

use byteorder::ReadBytesExt;

/// Labels are the individual nodes or components of a [DomainName]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Label(String);

impl Label {
    /// The maximum size of a single label within a domain name
    pub const MAX_LABEL_SIZE: usize = 63;

    fn into_bytes(self) -> Vec<u8> {
        let size = self.0.len();
        let mut buf = self.0.into_bytes();
        buf.splice(0..0, [size as u8]);
        buf
    }

    fn read_label(bytes: &mut Cursor<&[u8]>, dest: &mut [u8]) -> LabelResult<Self> {
        bytes.read_exact(dest).map_err(LabelError::Io)?;
        let label = std::str::from_utf8(dest)
            .map_err(LabelError::Convert)?
            .to_string();
        Ok(Self(label))
    }

    fn from_bytes(bytes: &mut Cursor<&[u8]>) -> LabelResult<Self> {
        let size = bytes.read_u8().map_err(LabelError::Io)?;
        // TODO check for msg compression (11 in high bits)

        let mut buf = vec![0u8; size as usize];
        Self::read_label(bytes, &mut buf)
    }

    fn from_bytes_with(bytes: &mut Cursor<&[u8]>, dest: &mut [u8]) -> LabelResult<Self> {
        let size = bytes.read_u8().map_err(LabelError::Io)?;
        // TODO check for msg compression (11 in high bits)

        let buf = &mut dest[..size as usize];
        Self::read_label(bytes, buf)
    }
}

/// [LabelError] wraps the errors that may be encountered during byte decoding of a [Label]
#[derive(Debug)]
pub enum LabelError {
    /// Stores an error encountered while using [std::io] traits and structs
    Io(std::io::Error),
    /// Stores an error encountered while converting from a sequence of [u8] to [String]
    Convert(std::str::Utf8Error),
}

impl std::fmt::Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelError::Io(io) => write!(f, "IO error: {io}"),
            LabelError::Convert(convert) => write!(f, "String parsing error: {convert}"),
        }
    }
}

type LabelResult<T> = std::result::Result<T, LabelError>;

/// Domain names define a name of a node in requests and responses
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(Vec<Label>);

impl DomainName {
    /// The maximum number of octets that represent a domain name (i.e., the sum of all label octets and label lengths)
    pub const MAX_NAME_SIZE: usize = 255;
    pub const MAX_UDP_MSG_SIZE: usize = 512;
    /// a domain name is terminated by a length byte of zero
    pub const TERMINATOR: u8 = 0;
}

impl From<String> for DomainName {
    fn from(mut value: String) -> Self {
        // split will have an empty string if '.' at end
        // leads to automatic dname delimter
        value.push('.');
        Self(
            value
                .split('.')
                .map(|substr| Label(substr.to_string()))
                .collect(),
        )
    }
}

impl DomainName {
    /// Converts a [DomainName] to owned bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0
            .iter()
            .flat_map(|label| Label::into_bytes(label.clone()))
            .collect()
    }

    /// Reads a [DomainName] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        // buffers and metadata storage

        let mut label_bytes_buffer = [0u8; Label::MAX_LABEL_SIZE];
        let mut labels = Vec::new();

        while bytes.position() < bytes.get_ref().len() as u64 {
            let label = Label::from_bytes_with(bytes, &mut label_bytes_buffer)
                .map_err(DomainNameError::Label)?;

            // Check for name delimiter
            if !label.0.is_empty() {
                labels.push(label);
            } else {
                labels.push(label);
                break;
            }
        }

        Ok(Self(labels))
    }

    /// Creates a new [DomainName]
    pub fn new(domain_name: &str) -> Self {
        DomainName::from(domain_name.to_string())
    }
}

type Result<T> = std::result::Result<T, DomainNameError>;

/// [DomainNameError] wraps the errors that may be encountered during byte decoding of a [DomainName]
#[derive(Debug)]
pub enum DomainNameError {
    /// Stores an error encountered while [Label] parsing
    Label(LabelError),
}

impl std::fmt::Display for DomainNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainNameError::Label(label) => write!(f, "Label parsing error: {label}"),
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
        let mut google_domain_bytes = Cursor::new(&b"\x06google\x03com\x00"[..]);

        let result_dname = DomainName::from_bytes(&mut google_domain_bytes)?;

        assert_eq!(result_dname, correct_dname);

        Ok(())
    }
}
