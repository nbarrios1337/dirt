use rand::Rng;

use crate::{dname::DomainName, header::Header, qclass::QClass, qtype::QType, question::Question};

#[derive(Debug, Clone)]
pub struct Query {
    header: Header,
    question: Question,
}

impl Query {
    /// Creates a new [`Query`] for available records of the specified type, for the specified domain name.
    pub fn new(domain_name: &str, record_type: QType, flags: u16) -> Self {
        let id: u16 = rand::thread_rng().gen();
        let header = Header {
            id,
            flags,
            num_questions: 1,
            num_answers: 0,
            num_authorities: 0,
            num_additionals: 0,
        };

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
        let mut buf = Vec::with_capacity(header_bytes.len() + question_bytes.len());
        buf.append(&mut header_bytes);
        buf.append(&mut question_bytes);
        buf
    }
}
