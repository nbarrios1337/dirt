use rand::Rng;

use crate::{
    dname::DomainName,
    header::{Header, HeaderFlags},
    qclass::QClass,
    qtype::QType,
    question::Question,
};

#[derive(Debug, Clone)]
pub struct Query {
    header: Header,
    question: Question,
}

impl Query {
    /// Creates a new [`Query`] for available records of the specified type, for the specified domain name.
    pub fn new(domain_name: &str, record_type: QType, flags: u16) -> Self {
        let id: u16 = rand::thread_rng().gen();
        let header = Header::new(id, HeaderFlags::try_from(flags).unwrap());

        let name = DomainName::new(domain_name);
        let question = Question {
            qname: name,
            qclass: QClass::IN,
            qtype: record_type,
        };
        Self { header, question }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut header_bytes = self.header.into_bytes();
        let mut question_bytes = self.question.into_bytes();

        header_bytes.append(&mut question_bytes);
        header_bytes
    }
}
