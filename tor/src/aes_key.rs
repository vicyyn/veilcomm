#[derive(Debug, Copy, Clone)]
pub struct AESKey([u8; 16]);

impl From<&[u8]> for AESKey {
    fn from(value: &[u8]) -> Self {
        Self(value.try_into().unwrap())
    }
}

impl AESKey {
    pub fn get_key(&self) -> [u8; 16] {
        self.0
    }
}
