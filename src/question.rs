#[derive(Debug, Clone)]
pub struct DomainName(String);

impl From<String> for DomainName {
    fn from(value: String) -> Self {
        DomainName(value)
    }
}

impl DomainName {
    pub fn encode_dns_name(&self) -> Vec<u8> {
        let mut encoded: Vec<u8> = self
            .0
            .split('.')
            .map(|substr| (substr.len(), substr.to_string()))
            .flat_map(|(len, mut substr)| {
                substr.insert(0, len as u8 as char);
                substr.into_bytes()
            })
            .collect();
        encoded.push(0);
        encoded
    }

    pub fn new(domain_name: &str) -> Self {
        DomainName(domain_name.to_string())
    }
}
#[derive(Debug, Clone)]
pub struct Question {
    pub qname: DomainName,
    pub qclass: u16,
    pub qtype: crate::qtype::QType,
}

impl Question {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = self.qname.encode_dns_name();
        let qtype_u16: u16 = self.qtype.into();
        buf.extend_from_slice(&qtype_u16.to_be_bytes());
        buf.extend_from_slice(&self.qclass.to_be_bytes());
        buf
    }
}
