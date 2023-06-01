#[derive(Debug, Clone, Copy)] // TODO what other derives needed?
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16, // TODO bitflags?
    pub num_questions: u16,
    pub num_answers: u16,
    pub num_authorities: u16,
    pub num_additionals: u16,
}

impl DnsHeader {
    pub fn to_bytes(self) -> Vec<u8> {
        // 6 fields, 2 bytes each
        let mut buf: Vec<u8> = Vec::with_capacity(6 * 2);
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&self.flags.to_be_bytes());
        buf.extend_from_slice(&self.num_questions.to_be_bytes());
        buf.extend_from_slice(&self.num_answers.to_be_bytes());
        buf.extend_from_slice(&self.num_authorities.to_be_bytes());
        buf.extend_from_slice(&self.num_additionals.to_be_bytes());
        buf
    }
}
