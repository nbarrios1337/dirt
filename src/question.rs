#[derive(Debug, Clone)]
pub struct Question {
    pub qname: crate::dname::DomainName,
    pub qclass: crate::qclass::QClass,
    pub qtype: crate::qtype::QType,
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
}
