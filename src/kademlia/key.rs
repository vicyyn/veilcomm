use crate::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Key(pub [u8; KEY_LEN]);

impl Key {
    pub fn new(key: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        let mut hash = [0; KEY_LEN];
        for i in 0..KEY_LEN {
            hash[i] = result[i];
        }
        Self(hash)
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x).expect("[FAILED] Key::Debug --> Failed to format contents of Key");
        }
        Ok(())
    }
}
