//! Domain names in messages are expressed in terms of a sequence of labels.
//!
//! See more in [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034)
//! and [RFC 1035 section 3.1](https://datatracker.ietf.org/doc/html/rfc1035#section-3.1)

use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::ReadBytesExt;

use thiserror::Error;

/// Labels are the individual nodes or components of a [`DomainName`]
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
        bytes.read_exact(dest).map_err(|source| {
            let mut owned_bytes = Cursor::new(bytes.get_ref().to_vec());
            owned_bytes.set_position(bytes.position());
            LabelError::Io {
                src_bytes: owned_bytes,
                dest_amt: dest.len(),
                source,
            }
        })?;
        let label = std::str::from_utf8(dest)
            .map_err(|source| LabelError::Convert {
                bytes: dest.to_vec(),
                source,
            })?
            .to_string();
        Ok(Self(label))
    }
}

/// [`LabelError`] wraps the errors that may be encountered during byte decoding of a [`Label`]
#[derive(Debug, Error)]
pub enum LabelError {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to read {dest_amt} bytes from {src_bytes:?}:\n\t{source}")]
    Io {
        src_bytes: Cursor<Vec<u8>>,
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

type LabelResult<T> = std::result::Result<T, LabelError>;

/// Domain names define a name of a node in requests and responses
#[derive(Clone, PartialEq, Eq)]
pub struct DomainName(Vec<Label>);

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

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        Self(
            value
                .split('.')
                .map(|substr| Label(substr.to_string()))
                .collect(),
        )
    }
}

impl From<DomainName> for String {
    fn from(value: DomainName) -> Self {
        value
            .0
            .into_iter()
            .map(|label| label.0)
            .reduce(|acc, label_str| acc + "." + &label_str)
            .unwrap_or_default()
    }
}

impl DomainName {
    /// Converts a [`DomainName`] to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        let mut val: Vec<u8> = self.0.into_iter().flat_map(Label::into_bytes).collect();
        // name bytes have zero octet delimiter
        val.push(DomainName::TERMINATOR);
        val
    }

    pub const fn is_compressed(size: u8) -> bool {
        size & 0b1100_0000 == 0b1100_0000
    }

    fn read_compressed_label(bytes: &mut Cursor<&[u8]>, size: u8) -> Result<Vec<Label>> {
        // get pointed-to name
        let second = bytes.read_u8()?;
        let name_pos = u16::from_be_bytes([size & 0b0011_1111, second]);

        // save current pos
        let old_pos = bytes.position();

        // get name
        bytes.seek(SeekFrom::Start(name_pos as u64))?;
        let name = DomainName::from_bytes(bytes)?;

        // reset to current pos
        bytes.seek(SeekFrom::Start(old_pos))?;
        Ok(name.0)
    }

    /// Reads a [`DomainName`] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        // buffers and metadata storage

        let mut label_bytes_buffer = [0u8; Label::MAX_LABEL_SIZE];
        let mut labels = Vec::new();

        loop {
            let size = bytes.read_u8()?;

            match size {
                size if Self::is_compressed(size) => {
                    labels.extend(Self::read_compressed_label(bytes, size)?);
                    break;
                }
                DomainName::TERMINATOR => {
                    break;
                }
                _ => {
                    let dest = &mut label_bytes_buffer[..size as usize];
                    let label = Label::read_label(bytes, dest)
                        .map_err(|source| DomainNameError::Label { size, source })?;
                    labels.push(label);
                }
            }
        }

        Ok(Self(labels))
    }

    /// Creates a new [`DomainName`]
    pub fn new(domain_name: &str) -> Self {
        DomainName::from(domain_name.to_string())
    }
}

type Result<T> = std::result::Result<T, DomainNameError>;

/// [`DomainNameError`] wraps the errors that may be encountered during byte decoding of a [`DomainName`]
#[derive(Debug, Error)]
pub enum DomainNameError {
    /// Stores an error encountered while [Label] parsing
    #[error("Attempted to parse a {size}-octets label:\n\t{source}")]
    Label {
        size: u8,
        #[source]
        source: LabelError,
    },
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse domain name data:\n\t{0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests encoding of "google.com"
    #[test]
    fn encode_dname() {
        let correct_bytes = b"\x06google\x03com\x00";

        let google_domain = DomainName::new("google.com");
        let result_bytes = google_domain.into_bytes();

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
