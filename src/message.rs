use std::io::Cursor;

use thiserror::Error;

use crate::{
    header::{Header, HeaderError},
    question::{Question, QuestionError},
    record::{Record, RecordError},
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

impl Message {
    /// Reads a [Message] a sequence of bytes
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> MessageResult<Self> {
        let header = Header::from_bytes(bytes)?;

        let questions: Vec<Question> = std::iter::repeat_with(|| Question::from_bytes(bytes))
            .take(header.num_questions as usize)
            .collect::<Result<Vec<Question>, QuestionError>>()?;

        let answers: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_answers as usize)
            .collect::<Result<Vec<Record>, RecordError>>()?;

        let authorities: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_authorities as usize)
            .collect::<Result<Vec<Record>, RecordError>>()?;

        let additionals: Vec<Record> = std::iter::repeat_with(|| Record::from_bytes(bytes))
            .take(header.num_additionals as usize)
            .collect::<Result<Vec<Record>, RecordError>>()?;

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

/// [MessageError] wraps the errors that may be encountered during byte decoding of a [Message]
#[derive(Debug, Error)]
pub enum MessageError {
    /// Stores an error encountered while using [std::io] traits and structs
    #[error("Failed to parse message data: {0}")]
    Io(#[from] std::io::Error),
    /// Encountered during header parsing
    #[error(transparent)]
    Header(#[from] HeaderError),
    /// Encountered during question parsing
    #[error(transparent)]
    Question(#[from] QuestionError),
    /// Encountered during record parsing
    #[error(transparent)]
    Record(#[from] RecordError),
}

type MessageResult<T> = std::result::Result<T, MessageError>;
