use std::io::{Cursor, Read};

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::{dname::DomainName, qclass::QClass, qtype::QType};

/// A resource record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// a domain name to which this resource record pertains.
    name: DomainName,
    /// This field specifies the meaning of the data in the RDATA field.
    qtype: QType,
    /// the class of the data in the RDATA field
    class: QClass,
    /// specifies the time interval (in seconds) that the resource record may be cached before it should be discarded.
    ///
    /// Zero values are interpreted to mean that the RR can only be used for the transaction in progress, and should not be cached.
    time_to_live: u32,
    /// a variable length string of octets that describes the resource. The format of this information varies according to the TYPE and CLASS of the resource record.
    ///
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    rdata: Vec<u8>,
}

impl Record {
    /// Reads a [Record] from a slice of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let qname = DomainName::from_bytes(bytes).map_err(RecordError::Name)?;

        let qtype = QType::try_from(bytes.read_u16::<NetworkEndian>().map_err(RecordError::Io)?)
            .map_err(RecordError::Type)?;

        let qclass = QClass::try_from(bytes.read_u16::<NetworkEndian>().map_err(RecordError::Io)?)
            .map_err(RecordError::Class)?;

        let ttl = bytes.read_u32::<NetworkEndian>().map_err(RecordError::Io)?;

        let data_length = bytes.read_u16::<NetworkEndian>().map_err(RecordError::Io)?;

        let mut data = vec![0; data_length as usize];
        bytes.read_exact(&mut data).map_err(RecordError::Io)?;

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
#[derive(Debug)]
pub enum RecordError {
    /// Stores an error encountered while using [std::io] traits and structs
    Io(std::io::Error),
    /// Stores an error encountered while parsing the [DomainName]
    Name(crate::dname::DomainNameError),
    /// Stores an error encountered while parsin the [QType]
    Type(num_enum::TryFromPrimitiveError<QType>),
    /// Stores an error encountered while parsin the [QClass]
    Class(num_enum::TryFromPrimitiveError<QClass>),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::Io(e) => write!(f, "IO parsing error: {e}"),
            RecordError::Name(e) => write!(f, "DomainName parsing error: {e}"),
            RecordError::Type(e) => write!(f, "Type parsing error: {e}"),
            RecordError::Class(e) => write!(f, "Class parsing error: {e}"),
        }
    }
}

impl std::error::Error for RecordError {}

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
