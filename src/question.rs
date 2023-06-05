use crate::{
    dname::{DomainName, DomainNameError},
    qclass::QClass,
    qtype::QType,
};

#[derive(Debug, Clone)]
pub struct Question {
    pub qname: DomainName,
    pub qtype: QType,
    pub qclass: QClass,
}

impl Question {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.qname.encode_dns_name();

        let qtype_u16: u16 = self.qtype.into();
        buf.extend_from_slice(&qtype_u16.to_be_bytes());

        let qclass_u16: u16 = self.qclass.into();
        buf.extend_from_slice(&qclass_u16.to_be_bytes());

        buf
    }

    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self> {
        use std::io::{BufRead, Read};

        // set up owned buffer for domain name parsing
        let mut question_bytes = Vec::with_capacity(DomainName::MAX_NAME_SIZE);

        // TODO use for question size?
        // Read all the name-related bytes (delimited by zero octet)
        let question_size = bytes
            .read_until(DomainName::TERMINATOR, &mut question_bytes)
            .map_err(QuestionError::Io)?;
        let qname = DomainName::decode_dns_name(&question_bytes).map_err(QuestionError::Name)?;

        // reusable buffer for u16 parsing
        let mut u16_buffer = [0u8; 2];

        // qtype parsing
        bytes
            .read_exact(&mut u16_buffer)
            .map_err(QuestionError::Io)?;
        let qtype = QType::try_from(u16::from_be_bytes(u16_buffer)).map_err(QuestionError::Type)?;

        // qclass parsing
        bytes
            .read_exact(&mut u16_buffer)
            .map_err(QuestionError::Io)?;
        let qclass =
            QClass::try_from(u16::from_be_bytes(u16_buffer)).map_err(QuestionError::Class)?;

        Ok(Self {
            qname,
            qtype,
            qclass,
        })
    }
}

type Result<T> = std::result::Result<T, QuestionError>;

#[derive(Debug)]
pub enum QuestionError {
    Io(std::io::Error),
    Name(DomainNameError),
    Type(num_enum::TryFromPrimitiveError<QType>),
    Class(num_enum::TryFromPrimitiveError<QClass>),
}

impl std::fmt::Display for QuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Name(e) => write!(f, "Name parsing error: {e}"),
            Self::Type(e) => write!(f, "type parsing error: {e}"),
            Self::Class(e) => write!(f, "class parsing error: {e}"),
        }
    }
}

impl std::error::Error for QuestionError {}
