#[derive(Debug, Clone, Copy)] // TODO what other derives needed?
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16, // TODO bitflags?
    pub num_questions: u16,
    pub num_answers: u16,
    pub num_authorities: u16,
    pub num_additionals: u16,
}

impl DnsHeader {
    pub fn to_bytes(self) -> Vec<u8> {
        // 6 fields, 2 bytes each
        let mut buf: Vec<u8> = Vec::with_capacity(6 * 2);
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&self.num_questions.to_be_bytes());
        buf.extend_from_slice(&self.num_answers.to_be_bytes());
        buf.extend_from_slice(&self.num_authorities.to_be_bytes());
        buf.extend_from_slice(&self.num_additionals.to_be_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        // header is only twelve bytes
        let buf: [u8; 12] = bytes[..12]
            .try_into()
            .expect("Not enough bytes of header information");

        let id = u16::from_be_bytes(buf[0..2].try_into().unwrap());
        let flags = u16::from_be_bytes(buf[2..4].try_into().unwrap());
        let num_questions = u16::from_be_bytes(buf[4..6].try_into().unwrap());
        let num_answers = u16::from_be_bytes(buf[6..8].try_into().unwrap());
        let num_authorities = u16::from_be_bytes(buf[8..10].try_into().unwrap());
        let num_additionals = u16::from_be_bytes(buf[10..12].try_into().unwrap());

        Self {
            id,
            flags,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DnsHeader;

    #[test]
    fn encode_header() {
        let header = DnsHeader {
            id: 0x1314,
            flags: 0,
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        };

        let header_bytes = header.to_bytes();

        let correct_bytes: Vec<u8> =
            vec![0x13, 0x14, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

        assert_eq!(header_bytes, correct_bytes);
    }

    #[test]
    fn decode_header() {
        let test_bytes = vec![
            0x82, 0x98, //id
            0x01, 0x00, //flags
            0x00, 0x01, // n_q
            0x00, 0x00, // n_ans
            0x00, 0x00, // n_auth
            0x00, 0x00, // n_add
        ];

        let expected_id = 0x8298;

        // recursion desired
        let expected_flags: u16 = 1 << 8;

        let result_header = DnsHeader::from_bytes(&test_bytes);

        assert_eq!(result_header.id, expected_id);
        assert_eq!(result_header.flags, expected_flags);
        assert_eq!(result_header.num_questions, 1);
        assert_eq!(result_header.num_answers, 0);
        assert_eq!(result_header.num_authorities, 0);
        assert_eq!(result_header.num_additionals, 0);
    }
}
