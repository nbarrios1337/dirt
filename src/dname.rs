#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(String);

impl DomainName {
    const MAX_LABEL_SIZE: usize = 63;
    const MAX_NAME_SIZE: usize = 255;
    const MAX_UDP_MSG_SIZE: usize = 512;
}

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName(value)
    }
}

impl DomainName {
    pub fn encode_dns_name(&self) -> Vec<u8> {
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

    pub fn decode_dns_name(bytes: &[u8]) -> Result<Self> {
        use std::io::prelude::*;

        let mut labels_metadata = Vec::new();
        let mut bytes_cursor = std::io::Cursor::new(bytes);

        let mut cur_length_slice = [0u8];
        // read positions and lengths per label
        while (bytes_cursor.position() as usize) < bytes.len() {
            // read into a single-byte slice
            bytes_cursor
                .read_exact(&mut cur_length_slice)
                .map_err(DomainNameError::Io)?;

            let cur_length = u8::from_be_bytes(cur_length_slice);

            // found the name delimiter
            if cur_length == 0 {
                break;
            }

            // TODO error check with MAX_LABEL_SIZE

            // store position and length for later use
            // the earlier single-byte read moved the cursor to the correct pos
            labels_metadata.push((bytes_cursor.position(), cur_length));

            // move to next length octet
            bytes_cursor
                .seek(std::io::SeekFrom::Current(cur_length as i64))
                .map_err(DomainNameError::Io)?;
        }

        // reset cursor for label reading
        bytes_cursor.set_position(0);

        let mut label_bytes_buffer = [0u8; Self::MAX_LABEL_SIZE];

        let labels: Vec<String> = labels_metadata
            .iter()
            .map(|(label_pos, label_length)| {
                let cur_label_bytes = &mut label_bytes_buffer[0..*label_length as usize];

                // move to start of label
                bytes_cursor.set_position(*label_pos);

                bytes_cursor
                    .read_exact(cur_label_bytes)
                    .map_err(DomainNameError::Io)?;

                let cur_label = std::str::from_utf8(cur_label_bytes)
                    .map_err(DomainNameError::Parse)?
                    .to_string();

                Ok(cur_label)
            })
            .collect::<Result<Vec<String>>>()?;

        Ok(Self(labels.join(".")))
    }

    pub fn new(domain_name: &str) -> Self {
        DomainName(domain_name.to_string())
    }
}

type Result<T> = std::result::Result<T, DomainNameError>;

#[derive(Debug)]
pub enum DomainNameError {
    Io(std::io::Error),
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
    fn qname_encoding() {
        let correct_bytes = b"\x06google\x03com\x00";

        let google_domain = DomainName::new("google.com");
        let result_bytes = google_domain.encode_dns_name();

        assert_eq!(result_bytes, correct_bytes);
    }

    /// Tests decoding of "google.com"
    #[test]
    fn qname_decoding() -> std::result::Result<(), DomainNameError> {
        let correct_dname = DomainName::new("google.com");
        let google_domain_bytes = b"\x06google\x03com\x00";

        let result_dname = DomainName::decode_dns_name(google_domain_bytes)?;

        assert_eq!(result_dname, correct_dname);

        Ok(())
    }
}
