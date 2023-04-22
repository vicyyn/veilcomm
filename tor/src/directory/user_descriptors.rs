use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDescriptors(Vec<UserDescriptor>);

impl UserDescriptors {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_user_descriptor(&self, address: [u8; 32]) -> Option<UserDescriptor> {
        self.0.clone().into_iter().find(|x| x.address.eq(&address))
    }

    pub fn add_user_descriptor(&mut self, user_descriptor: UserDescriptor) {
        self.0.push(user_descriptor);
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("[FAILED] UserDescriptors::serialize --> Unable to serialize")
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        bincode::deserialize(&buffer.to_vec())
            .expect("[FAILED] UserDescriptors::deserialize --> Unable to deserialize")
    }
}
