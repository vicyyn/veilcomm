use crate::*;

const KEY_LEN: usize = 32;

#[derive(Clone, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq, Copy)]
pub struct Distance(pub [u8; KEY_LEN]);

impl Distance {
    pub fn new(k1: &Key, k2: &Key) -> Distance {
        let mut ret = [0; KEY_LEN];
        for i in 0..KEY_LEN {
            ret[i] = k1.0[i] ^ k2.0[i];
        }
        Self(ret)
    }
}

impl Debug for Distance {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:X}", x)
                .expect("[FAILED] Distance::Debug --> Failed to format contents of Key");
        }
        Ok(())
    }
}

impl Binary for Distance {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for x in &self.0 {
            write!(f, "{:b}", x)
                .expect("[FAILED] Key::Binary --> Failed to format contents of Distance");
        }
        Ok(())
    }
}
