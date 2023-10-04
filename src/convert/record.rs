use std::io::{Cursor, Read};

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::{
    dname::DomainName,
    qclass::QClass,
    qtype::QType,
    record::{Record, Result},
};

impl Record {
    /// Reads a [`Record`] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let qname = DomainName::from_bytes(bytes)?;
        let qtype = QType::try_from(bytes.read_u16::<NetworkEndian>()?)?;
        let qclass = QClass::try_from(bytes.read_u16::<NetworkEndian>()?)?;
        let ttl = bytes.read_u32::<NetworkEndian>()?;

        let data_length = bytes.read_u16::<NetworkEndian>()?;

        let data = match qtype {
            QType::NS | QType::CNAME => String::from(DomainName::from_bytes(bytes)?).into_bytes(),
            QType::A => {
                let mut data = vec![0; data_length as usize];
                bytes.read_exact(&mut data)?;
                data[..4].to_vec()
            }
            _ => {
                let mut data = vec![0; data_length as usize];
                bytes.read_exact(&mut data)?;
                data
            }
        };

        Ok(Self {
            name: qname,
            qtype,
            class: qclass,
            time_to_live: ttl,
            rdata: data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_record() -> Result<()> {
        let record_bytes = b"`V\x81\x80\x00\x01\x00\x01\x00\x00\x00\x00\x03www\x07example\x03com\x00\x00\x01\x00\x01\xc0\x0c\x00\x01\x00\x01\x00\x00R\x9b\x00\x04]\xb8\xd8\"";
        let correct_record = Record {
            name: DomainName::new("www.example.com"),
            qtype: QType::A,
            class: QClass::IN,
            time_to_live: 21147,
            rdata: b"]\xb8\xd8\"".to_vec(),
        };

        let mut rec_bytes_reader = Cursor::new(&record_bytes[..]);
        let hdr = crate::header::Header::from_bytes(&mut rec_bytes_reader).unwrap();
        eprintln!("header: {hdr:?}");
        let q = crate::question::Question::from_bytes(&mut rec_bytes_reader).unwrap();
        eprintln!("question: {q:?}");
        let result_record = Record::from_bytes(&mut rec_bytes_reader)?;
        eprintln!("record: {result_record:?}");

        assert_eq!(result_record, correct_record);
        Ok(())
    }
}
