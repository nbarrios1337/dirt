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

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (qname, metadata_pos) =
            DomainName::decode_dns_name(bytes).map_err(QuestionError::Name)?;

        let bytes = &bytes[metadata_pos..];

        let qtype = <[u8; 2]>::try_from(&bytes[0..2])
            .map_err(QuestionError::Convert)
            .map(u16::from_be_bytes)
            .and_then(|val| match QType::try_from(val) {
                Ok(qtype) => Ok(qtype),
                Err(num_err) => Err(QuestionError::Type(num_err)),
            })?;

        let qclass = <[u8; 2]>::try_from(&bytes[2..4])
            .map_err(QuestionError::Convert)
            .map(u16::from_be_bytes)
            .and_then(|val| match QClass::try_from(val) {
                Ok(qclass) => Ok(qclass),
                Err(num_err) => Err(QuestionError::Class(num_err)),
            })?;

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
    Name(DomainNameError),
    Type(num_enum::TryFromPrimitiveError<QType>),
    Class(num_enum::TryFromPrimitiveError<QClass>),
    Convert(std::array::TryFromSliceError),
}

impl std::fmt::Display for QuestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuestionError::Name(e) => write!(f, "Name parsing error: {e}"),
            QuestionError::Type(e) => write!(f, "type parsing error: {e}"),
            QuestionError::Class(e) => write!(f, "class parsing error: {e}"),
            QuestionError::Convert(e) => write!(f, "byte slice to primitive conversion error: {e}"),
        }
    }
}

impl std::error::Error for QuestionError {}
