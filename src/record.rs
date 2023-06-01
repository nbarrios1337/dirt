use crate::{qclass::QClass, qtype::QType, question::DomainName};

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
