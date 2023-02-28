use std::convert::From;

#[derive(Debug)]
pub struct AESKey([u8; 16]);

impl From<&[u8]> for AESKey {
    fn from(value: &[u8]) -> Self {
        Self(value.try_into().unwrap())
    }
}
