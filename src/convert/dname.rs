use std::io::{Cursor, Seek, SeekFrom};

use byteorder::ReadBytesExt;

use crate::types::dname::*;

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        Self(
            value
                .split('.')
                .map(|substr| Label::new(substr.to_string()))
                .collect(),
        )
    }
}

impl From<DomainName> for String {
    fn from(value: DomainName) -> Self {
        value
            .0
            .into_iter()
            .map(|label| label.to_string())
            .reduce(|acc, label_str| acc + "." + &label_str)
            .unwrap_or_default()
    }
}

impl DomainName {
    /// Converts a [`DomainName`] to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        let mut val: Vec<u8> = self.0.into_iter().flat_map(Label::into_bytes).collect();
        // name bytes have zero octet delimiter
        val.push(Self::TERMINATOR);
        val
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
                    // get pointed-to name
                    let second = bytes.read_u8()?;
                    let name_pos = u16::from_be_bytes([size & 0b0011_1111, second]);
                    labels.extend(DomainName::read_compressed_label(bytes, name_pos)?);
                    break;
                }
                Self::TERMINATOR => {
                    break;
                }
                _ => {
                    let dest = &mut label_bytes_buffer[..size as usize];
                    let label = Label::read_label(bytes, dest)
                        .map_err(|source| Error::Label { size, source })?;
                    labels.push(label);
                }
            }
        }

        Ok(Self(labels))
    }

    fn read_compressed_label(bytes: &mut Cursor<&[u8]>, name_pos: u16) -> Result<Vec<Label>> {
        // save current pos
        let old_pos = bytes.position();

        // get name
        bytes.seek(SeekFrom::Start(name_pos as u64))?;
        let name = Self::from_bytes(bytes)?;

        // reset to current pos
        bytes.seek(SeekFrom::Start(old_pos))?;
        Ok(name.0)
    }
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
