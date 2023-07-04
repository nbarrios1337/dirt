use std::io::{Cursor, Read};

use byteorder::{NetworkEndian, ReadBytesExt};
use thiserror::Error;

use crate::{
    dname::{DomainName, DomainNameError},
    qclass::QClass,
    qtype::QType,
};

/// A resource record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// a domain name to which this resource record pertains.
    pub name: DomainName,
    /// This field specifies the meaning of the data in the RDATA field.
    pub qtype: QType,
    /// the class of the data in the RDATA field
    pub class: QClass,
    /// specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    ///
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    pub time_to_live: u32,
    /// a variable length string of octets that describes the resource. The format of this information varies according to the TYPE and CLASS of the resource record.
    ///
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    pub rdata: Vec<u8>,
}

impl Record {
    /// Reads a [Record] from a slice of bytes
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

type Result<T> = std::result::Result<T, RecordError>;

/// [RecordError] wraps the errors that may be encountered during byte decoding of a [Record]
#[derive(Debug, Error)]
pub enum RecordError {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse question data: {0}")]
    Io(#[from] std::io::Error),
    /// Stores an error encountered while parsing the [DomainName]
    #[error(transparent)]
    Name(#[from] DomainNameError),
    /// Stores an error encountered while parsin the [QType]
    #[error("Failed to convert primitive to QType: {0}")]
    Type(#[from] num_enum::TryFromPrimitiveError<QType>),
    /// Stores an error encountered while parsin the [QClass]
    #[error("Failed to convert primitive to QClass: {0}")]
    Class(#[from] num_enum::TryFromPrimitiveError<QClass>),
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
        eprintln!("{hdr:?}");
        let q = crate::question::Question::from_bytes(&mut rec_bytes_reader).unwrap();
        eprintln!("{q:?}");
        let result_record = Record::from_bytes(&mut rec_bytes_reader)?;

        assert_eq!(result_record, correct_record);
        Ok(())
    }
}
