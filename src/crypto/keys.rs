use openssl::{dh::Dh, pkey::Private, rsa::Rsa};

pub struct Keys {
    pub rsa_private: Rsa<Private>,
    pub dh: Dh<Private>,
}
