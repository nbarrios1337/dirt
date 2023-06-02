#[derive(Debug, Clone)]
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

    pub fn decode_dns_name(mut bytes: &[u8]) -> Result<Self> {
        use std::io::prelude::*;
        let mut labels = Vec::new();

        let mut label_bytes_buffer: Vec<u8> = Vec::with_capacity(Self::MAX_LABEL_SIZE);

        // while the length octet is not zero
        while bytes[0] != 0 {
            let cur_length: usize = bytes[0] as usize;
            // get exactly the amt of bytes for the current label
            bytes
                .read_exact(&mut label_bytes_buffer[..cur_length])
                .map_err(DomainNameError::Io)?;

            let bytes_to_convert = &label_bytes_buffer[..cur_length];

            let cur_label =
                std::str::from_utf8(bytes_to_convert).map_err(DomainNameError::Parse)?;
            labels.push(cur_label);
        }

        Ok(Self(labels.join(".")))
    }

    pub fn new(domain_name: &str) -> Self {
        DomainName(domain_name.to_string())
    }
}

type Result<T> = std::result::Result<T, DomainNameError>;

#[derive(Debug)]
enum DomainNameError {
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
}
