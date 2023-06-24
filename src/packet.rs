use std::io::Cursor;

use thiserror::Error;

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
#[derive(Debug, Error)]
pub enum PacketError {
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

type PacketResult<T> = std::result::Result<T, PacketError>;
