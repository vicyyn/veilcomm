#[derive(Debug)]
pub enum DirectoryEvent {
    AddUserDescriptor = 0,
    AddRelay = 1,
    GetUserDescriptors = 2,
    GetRelays = 3,

    AddedUserDescriptor = 4,
    AddedRelay = 5,
}

impl DirectoryEvent {
    pub fn serialize(&self) -> u8 {
        match self {
            DirectoryEvent::AddUserDescriptor => 0,
            DirectoryEvent::AddRelay => 1,
            DirectoryEvent::GetUserDescriptors => 2,
            DirectoryEvent::GetRelays => 3,
            DirectoryEvent::AddedUserDescriptor => 4,
            DirectoryEvent::AddedRelay => 5,
        }
    }

    pub fn deserialize(value: u8) -> Self {
        match value {
            0 => DirectoryEvent::AddUserDescriptor,
            1 => DirectoryEvent::AddRelay,
            2 => DirectoryEvent::GetUserDescriptors,
            3 => DirectoryEvent::GetRelays,
            4 => DirectoryEvent::AddedUserDescriptor,
            5 => DirectoryEvent::AddedRelay,
            _ => panic!("[FAILED] DirectoryEvent::deserialize --> Invalid value"),
        }
    }
}
