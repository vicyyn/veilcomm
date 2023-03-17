use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Error, Formatter};

const ID_LEN: usize = 32;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Id(pub [u8; ID_LEN]);

impl Id {
    pub fn new(id: String) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        let result = hasher.finalize();
        let mut hash = [0; ID_LEN];
        for i in 0..ID_LEN {
            hash[i] = result[i];
        }
        Self(hash)
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x).expect("[FAILED] Id::Debug --> Failed to format contents of Id");
        }
        Ok(())
    }
}
