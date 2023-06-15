use std::io::Cursor;

use crate::{
    header::{Header, HeaderError},
    question::{Question, QuestionError},
    record::{Record, RecordError},
};

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl Packet {
    pub fn from_bytes(bytes: &mut Cursor<&[u8]>) -> PacketResult<Self> {
        let header = Header::from_bytes(bytes).map_err(PacketError::Header)?;

        let questions: Vec<Question> =
            std::iter::repeat_with(|| Question::from_bytes(bytes).map_err(PacketError::Question))
                .take(header.num_questions as usize)
                .collect::<PacketResult<Vec<Question>>>()?;

        let answers: Vec<Record> =
            std::iter::repeat_with(|| Record::from_bytes(bytes).map_err(PacketError::Record))
                .take(header.num_answers as usize)
                .collect::<PacketResult<Vec<Record>>>()?;

        let authorities: Vec<Record> =
            std::iter::repeat_with(|| Record::from_bytes(bytes).map_err(PacketError::Record))
                .take(header.num_authorities as usize)
                .collect::<PacketResult<Vec<Record>>>()?;

        let additionals: Vec<Record> =
            std::iter::repeat_with(|| Record::from_bytes(bytes).map_err(PacketError::Record))
                .take(header.num_additionals as usize)
                .collect::<PacketResult<Vec<Record>>>()?;

        Ok(Self {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }
}

/// [PacketError] wraps the errors that may be encountered during byte decoding of a [Packet]
#[derive(Debug)]
pub enum PacketError {
    Header(HeaderError),
    Question(QuestionError),
    Record(RecordError),
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::Header(e) => write!(f, "Header parsing error: {e}"),
            PacketError::Question(e) => write!(f, "Question parsing error: {e}"),
            PacketError::Record(e) => write!(f, "Record parsing error: {e}"),
        }
    }
}

type PacketResult<T> = std::result::Result<T, PacketError>;
