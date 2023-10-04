use crate::{
    dname::DomainName, header::Header, qclass::QClass, qtype::QType, question::Question,
    record::Record,
};

/// All communications inside of the domain protocol are carried in a single format called a message.
///
/// The top level format of message is divided
///  into 5 sections (some of which are empty in certain cases) shown below:
/// ```text
/// +---------------------+
/// |        Header       |
/// +---------------------+
/// |       Question      | the question for the name server
/// +---------------------+
/// |        Answer       | RRs answering the question
/// +---------------------+
/// |      Authority      | RRs pointing toward an authority
/// +---------------------+
/// |      Additional     | RRs holding additional information
/// +---------------------+
/// ```
#[derive(Debug)]
pub struct Message {
    pub header: Header,
    /// The query name(s) and other query parameters.
    pub questions: Vec<Question>,
    /// RRs which directly answer the query.
    pub answers: Vec<Record>,
    /// RRs which describe other authoritative servers.
    ///
    /// May optionally carry the SOA RR for the authoritative data in the answer section
    pub authorities: Vec<Record>,
    /// RRs which may be helpful in using the RRs in the other sections.
    pub additionals: Vec<Record>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MsgSection {
    Answers,
    Authorities,
    Additionals,
}

// ctors
impl Message {
    /// Creates a new [`Message`] containing a single [`Question`]
    pub fn new_query(
        domain_name: &str,
        record_type: QType,
        authoritative: bool,
        recursion_desired: bool,
    ) -> Self {
        let header = Header::gen_query_header(0, authoritative, recursion_desired).unwrap();
        let name = DomainName::new(domain_name);
        let question = Question {
            qname: name,
            qclass: QClass::IN,
            qtype: record_type,
        };

        Self {
            header,
            questions: vec![question],
            answers: vec![],
            authorities: vec![],
            additionals: vec![],
        }
    }
}

// querying data
impl Message {
    pub fn get_records(&self, section: MsgSection) -> &[Record] {
        match section {
            MsgSection::Answers => &self.answers,
            MsgSection::Authorities => &self.authorities,
            MsgSection::Additionals => &self.additionals,
        }
    }

    pub fn get_record_by_type_from(&self, qtype: QType, section: MsgSection) -> Option<&Record> {
        self.get_records(section)
            .iter()
            .find(|rec| rec.qtype == qtype)
    }
}

/// Wraps the errors that may be encountered during byte decoding of a [`Message`]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse message data: {0}")]
    Io(#[from] std::io::Error),
    /// Encountered during header parsing
    #[error(transparent)]
    Header(#[from] crate::header::Error),
    /// Encountered during question parsing
    #[error(transparent)]
    Question(#[from] crate::question::Error),
    /// Encountered during record parsing
    #[error(transparent)]
    Record(#[from] crate::record::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
