use rand::{thread_rng, Rng};

pub fn generate_random_aes_key() -> [u8; 16] {
    let mut rand = thread_rng();
    rand.gen::<[u8; 16]>()
}
