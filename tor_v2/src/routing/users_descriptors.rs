use directory::UserDescriptor;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct _UserDescriptors(Vec<UserDescriptor>);

impl _UserDescriptors {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn new_from(user_descriptors: Vec<UserDescriptor>) -> Self {
        Self(user_descriptors)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_user_descriptors(&self) -> Vec<UserDescriptor> {
        self.0.clone()
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

    pub fn set(&mut self, user_descriptors: Vec<UserDescriptor>) {
        self.0.clear();
        for user_descriptor in user_descriptors {
            self.0.push(user_descriptor);
        }
    }
}

pub struct UserDescriptors(Arc<RwLock<_UserDescriptors>>);

impl UserDescriptors {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(_UserDescriptors::new())))
    }

    pub fn new_from(user_descriptors: Vec<UserDescriptor>) -> Self {
        Self(Arc::new(RwLock::new(_UserDescriptors::new_from(
            user_descriptors,
        ))))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    pub fn get_user_descriptor(&self, address: [u8; 32]) -> Option<UserDescriptor> {
        self.0
            .read()
            .unwrap()
            .0
            .clone()
            .into_iter()
            .find(|x| x.address.eq(&address))
    }

    pub fn get_user_descriptors(&self) -> Vec<UserDescriptor> {
        self.0.read().unwrap().get_user_descriptors()
    }

    pub fn add_user_descriptor(&self, user_descriptor: UserDescriptor) {
        self.0.write_all().unwrap().0.push(user_descriptor);
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.0.read().unwrap().serialize()
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        Self(Arc::new(RwLock::new(_UserDescriptors::deserialize(buffer))))
    }

    pub fn set(&self, user_descriptors: Self) {
        self.0
            .write_all()
            .unwrap()
            .set(user_descriptors.get_user_descriptors());
    }
}
