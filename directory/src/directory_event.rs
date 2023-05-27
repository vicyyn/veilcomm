use strum_macros::FromRepr;

#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub enum DirectoryEvent {
    AddUserDescriptor = 0,
    AddRelay = 1,
    GetUserDescriptors = 2,
    GetRelays = 3,

    AddedUserDescriptor = 4,
    AddedRelay = 5,
}
