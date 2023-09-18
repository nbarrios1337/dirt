use std::io::Cursor;

use byteorder::{ByteOrder, NetworkEndian, ReadBytesExt};

use crate::header::{Header, HeaderFlags, OpCode, ResponseCode, Result};

impl HeaderFlags {
    fn as_u16(&self) -> u16 {
        // first u8
        let higher: u8 = (self.query_response as u8) << 7
            | u8::from(self.op_code) << 3
            | (self.auth_answer as u8) << 2
            | (self.truncated as u8) << 1
            | self.recursion_desired as u8;

        let lower: u8 = (self.recursion_avail as u8) << 7 | u8::from(self.response_code);

        debug_assert_eq!(
            self,
            &Self::from_u16(u16::from_be_bytes([higher, lower])).unwrap()
        );

        u16::from_be_bytes([higher, lower])
    }

    fn from_u16(bytes: u16) -> std::result::Result<Self, String> {
        let [higher, lower] = bytes.to_be_bytes();

        let query_response = (higher >> 7) & 1 != 0;
        let op_code = OpCode::try_from((higher & 0b0111_1000) >> 3)?;
        let auth_answer = (higher >> 2) & 1 != 0;
        let truncated = (higher >> 1) & 1 != 0;
        let recursion_desired = higher & 1 != 0;

        let recursion_avail = (lower >> 7) & 1 != 0;
        let response_code = ResponseCode::try_from(lower & 0b0111_1111)?;

        Ok(Self {
            query_response,
            op_code,
            auth_answer,
            truncated,
            recursion_desired,
            recursion_avail,
            response_code,
        })
    }
}

impl From<HeaderFlags> for u16 {
    fn from(value: HeaderFlags) -> Self {
        value.as_u16()
    }
}

impl TryFrom<u16> for HeaderFlags {
    type Error = String;

    fn try_from(value: u16) -> std::result::Result<Self, Self::Error> {
        Self::from_u16(value)
    }
}

impl Header {
    /// Convert a header to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        // 6 fields, 2 bytes each
        let mut buf: Vec<u8> = vec![0u8; 6 * std::mem::size_of::<u16>()];
        NetworkEndian::write_u16_into(
            &[
                self.id,
                u16::from(self.flags),
                self.num_questions,
                self.num_answers,
                self.num_authorities,
                self.num_additionals,
            ],
            &mut buf,
        );
        buf
    }

    /// Reads a header from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let mut buf = [0u16; 6];
        bytes.read_u16_into::<NetworkEndian>(&mut buf)?;
        let [id, flags, num_questions, num_answers, num_authorities, num_additionals]: [u16; 6] =
            buf;

        let flags = HeaderFlags::try_from(flags)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(Self {
            id,
            flags,
            num_questions,
            num_answers,
            num_authorities,
            num_additionals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_header() {
        let header = Header {
            id: 0x1314,
            flags: HeaderFlags::default(),
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        };

        let header_bytes = header.into_bytes();

        let correct_bytes: Vec<u8> =
            vec![0x13, 0x14, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

        assert_eq!(header_bytes, correct_bytes);
    }

    #[test]
    fn decode_header() -> Result<()> {
        let test_bytes = vec![
            0x82, 0x98, // id
            0x01, 0x00, // flags
            0x00, 0x01, // n_q
            0x00, 0x00, // n_ans
            0x00, 0x00, // n_auth
            0x00, 0x00, // n_add
        ];

        let expected_id = 0x8298;

        // recursion desired
        let expected_flags: u16 = 1 << 8;

        let expected_header =
            Header::new(expected_id, HeaderFlags::try_from(expected_flags).unwrap());

        let result_header = Header::from_bytes(&mut Cursor::new(&test_bytes))?;

        assert_eq!(result_header, expected_header);

        Ok(())
    }
}
