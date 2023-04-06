use network::Node;
use openssl::pkey::{PKey, Private, Public};

pub struct HiddenUserDescriptor {
    pub address: [u8; 32],
    pub publickey: PKey<Public>,
    pub introduction_points: Vec<Node>,
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_hidden_user_descriptor() {}
}
