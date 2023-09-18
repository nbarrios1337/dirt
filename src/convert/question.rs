use std::io::Cursor;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    dname::DomainName,
    qclass::QClass,
    qtype::QType,
    question::{Question, Result},
};

impl Question {
    /// Converts a [`Question`] to owned bytes
    pub fn into_bytes(self) -> Vec<u8> {
        let mut buf = self.qname.into_bytes();

        buf.write_u16::<NetworkEndian>(self.qtype.into()).unwrap();
        buf.write_u16::<NetworkEndian>(self.qclass.into()).unwrap();

        buf
    }

    /// Reads a [`Question`] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let qname = DomainName::from_bytes(bytes)?;
        let qtype = QType::try_from(bytes.read_u16::<NetworkEndian>()?)?;
        let qclass = QClass::try_from(bytes.read_u16::<NetworkEndian>()?)?;

        Ok(Self {
            qname,
            qtype,
            qclass,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_question() {
        let question = Question {
            qname: DomainName::new("google.com"),
            qclass: QClass::IN,
            qtype: QType::A,
        };

        let correct_bytes = b"\x06google\x03com\x00\x00\x01\x00\x01";
        let result_bytes = question.into_bytes();

        assert_eq!(result_bytes, correct_bytes);
    }

    #[test]
    fn decode_question() -> Result<()> {
        let correct_question = Question {
            qname: DomainName::new("google.com"),
            qclass: QClass::IN,
            qtype: QType::A,
        };

        let mut bytes = Cursor::new(&b"\x06google\x03com\x00\x00\x01\x00\x01"[..]);
        let result_question = Question::from_bytes(&mut bytes)?;

        assert_eq!(result_question, correct_question);
        Ok(())
    }
}
