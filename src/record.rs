use crate::{dname::DomainName, qclass::QClass, qtype::QType};

/// A resource record
#[derive(Debug, Clone)]
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
    /// specifies the length in octets of the RDATA field.
    data_length: u16,
    /// a variable length string of octets that describes the resource. The format of this information varies according to the TYPE and CLASS of the resource record.
    ///
    /// For example, the if the TYPE is A and the CLASS is IN, the RDATA field is a 4 octet ARPA Internet address.
    rdata: Vec<u8>,
}

impl Record {
    /// Reads a [Record] from a slice of bytes
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        use std::io::{BufRead, Read};

        // set up owned buffer for domain name parsing
        let mut question_bytes = Vec::with_capacity(DomainName::MAX_NAME_SIZE);

        // TODO use for question size?
        // Read all the name-related bytes (delimited by zero octet)
        let question_size = bytes
            .read_until(DomainName::TERMINATOR, &mut question_bytes)
            .map_err(RecordError::Io)?;
        let qname = DomainName::from_bytes(&mut &question_bytes[..]).map_err(RecordError::Name)?;

        // reusable buffer for u16 parsing
        let mut u16_buffer = [0u8; 2];
        let mut u32_buffer = [0u8; 4];

        // qtype parsing
        bytes.read_exact(&mut u16_buffer).map_err(RecordError::Io)?;
        let qtype = QType::try_from(u16::from_be_bytes(u16_buffer)).map_err(RecordError::Type)?;

        // qclass parsing
        bytes.read_exact(&mut u16_buffer).map_err(RecordError::Io)?;
        let qclass =
            QClass::try_from(u16::from_be_bytes(u16_buffer)).map_err(RecordError::Class)?;

        // ttl parsing
        bytes.read_exact(&mut u32_buffer).map_err(RecordError::Io)?;
        let ttl = u32::from_be_bytes(u32_buffer);

        // data length parsing
        bytes.read_exact(&mut u16_buffer).map_err(RecordError::Io)?;
        let data_length = u16::from_be_bytes(u16_buffer);

        // rdata parsing
        let mut data = vec![0; data_length as usize];
        bytes.read_exact(&mut data).map_err(RecordError::Io)?;

        Ok(Self {
            name: qname,
            qtype,
            class: qclass,
            time_to_live: ttl,
            data_length,
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
