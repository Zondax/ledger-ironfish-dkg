use crate::parser::KEY_LENGTH;

pub struct Epk<'a>(&'a [u8; KEY_LENGTH]);
