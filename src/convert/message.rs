use std::io::Cursor;

use crate::{
    header::Header,
    message::{Message, Result},
    question::Question,
    record::Record,
};

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

    /// Converts a query [`Message`] to owned bytes
    pub fn query_into_bytes(self) -> Vec<u8> {
        assert_eq!(self.questions.len(), 1);
        assert_eq!(self.answers.len(), 0);
        assert_eq!(self.authorities.len(), 0);
        assert_eq!(self.additionals.len(), 0);

        // header
        let mut buf = self.header.into_bytes();

        // questions
        buf.extend(self.questions.into_iter().flat_map(|q| q.into_bytes()));
        buf
    }
}
