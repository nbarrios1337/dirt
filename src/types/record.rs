use crate::{dname::DomainName, qclass::QClass, qtype::QType};

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

// parsing data
impl Record {
    pub fn data_as_str(&self) -> &str {
        std::str::from_utf8(&self.rdata).unwrap()
    }

    pub fn data_as_ip_addr(&self) -> std::net::IpAddr {
        match self.qtype {
            QType::A => std::net::IpAddr::V4(std::net::Ipv4Addr::from(
                <[u8; 4]>::try_from(&self.rdata[..4]).unwrap(),
            )),
            QType::AAAA => std::net::IpAddr::V6(std::net::Ipv6Addr::from(
                <[u8; 16]>::try_from(&self.rdata[..16]).unwrap(),
            )),
            _ => unreachable!(),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Wraps the errors that may be encountered during byte decoding of a [`Record`]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse question data: {0}")]
    Io(#[from] std::io::Error),
    /// Stores an error encountered while parsing the [DomainName]
    #[error(transparent)]
    Name(#[from] crate::dname::Error),
    /// Stores an error encountered while parsin the [QType]
    #[error("Failed to convert primitive to QType: {0}")]
    Type(#[from] num_enum::TryFromPrimitiveError<QType>),
    /// Stores an error encountered while parsin the [QClass]
    #[error("Failed to convert primitive to QClass: {0}")]
    Class(#[from] num_enum::TryFromPrimitiveError<QClass>),
}
