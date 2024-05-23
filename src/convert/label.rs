use std::io::{Cursor, Read};

use crate::types::dname::label::*;

impl Label {
    pub fn into_bytes(self) -> Vec<u8> {
        let size = self.0.len();
        let mut buf = self.0.into_bytes();
        buf.splice(0..0, [size as u8]);
        buf
    }

    pub fn read_label(bytes: &mut Cursor<&[u8]>, dest: &mut [u8]) -> Result<Self> {
        bytes.read_exact(dest).map_err(|source| Error::Io {
            src_amt: bytes.get_ref()[bytes.position() as usize..].len(),
            dest_amt: dest.len(),
            source,
        })?;
        let label = std::str::from_utf8(dest)
            .map_err(|source| Error::Convert {
                bytes: dest.to_vec(),
                source,
            })?
            .to_string();
        Ok(Self(label))
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
