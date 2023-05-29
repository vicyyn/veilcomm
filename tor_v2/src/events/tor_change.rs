use directory::{RelayDescriptor, UserDescriptor};

pub enum TorChange {
    ReceiveMessage((String, String)),
    SendMessage((String, String)),
    Logs(String),
    ReceiveRelays(Vec<RelayDescriptor>),
    ReceiveUsers(Vec<UserDescriptor>),
}
