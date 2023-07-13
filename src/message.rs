use std::io::Cursor;

use crate::{header::Header, qtype::QType, question::Question, record::Record};

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

#[derive(Debug, PartialEq, Eq)]
pub enum MsgSection {
    Answers,
    Authorities,
    Additionals,
}

impl Message {
    /// Reads a [`Message`] a sequence of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> Result<Self> {
        let header = Header::from_bytes(bytes)?;

        let questions: Vec<Question> = std::iter::repeat_with(|| Question::from_bytes(bytes))
            .take(header.num_questions as usize)
            .collect::<std::result::Result<Vec<Question>, crate::question::Error>>()?;

        let answers: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_answers as usize)
            .collect::<std::result::Result<Vec<Record>, crate::record::Error>>()?;

        let authorities: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_authorities as usize)
            .collect::<std::result::Result<Vec<Record>, crate::record::Error>>()?;

        let additionals: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_additionals as usize)
            .collect::<std::result::Result<Vec<Record>, crate::record::Error>>()?;

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
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
