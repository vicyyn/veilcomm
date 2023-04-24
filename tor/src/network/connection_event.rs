use crate::*;

pub enum ReadConnectionEvent {
    ReceiveUserDescriptor(UserDescriptor),
    ReceiveUserDescriptors(UserDescriptors),
    ReceiveRelay(Relay),
    ReceiveRelays(Relays),
    ReceiveCell(Cell),
}

pub enum WriteConnectionEvent {
    SendUserDescriptor(UserDescriptor),
    SendRelay(Relay),
    SendRelays(Relays),
    SendCell(Cell),
    SendUserDescriptors(UserDescriptors),
}
