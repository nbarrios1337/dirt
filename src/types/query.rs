use crate::{dname::DomainName, header::Header, qclass::QClass, qtype::QType, question::Question};

#[derive(Debug, Clone)]
pub struct Query {
    header: Header,
    question: Question,
}

impl Query {
    /// Creates a new [`Query`] for available records of the specified type, for the specified domain name.
    pub fn new(
        domain_name: &str,
        record_type: QType,
        authoritative: bool,
        recursion_desired: bool,
    ) -> Self {
        let header = Header::gen_query_header(0, authoritative, recursion_desired).unwrap();

        tracing::debug!("For {domain_name}, Generated header: {header:?}");

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
