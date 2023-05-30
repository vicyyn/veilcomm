use directory::RelayDescriptor;

pub enum TorChange {
    ReceiveMessage(String),
    SendMessage((String, String)),
    Logs(String),
    ReceiveRelays(Vec<RelayDescriptor>),
    Initialized(String),
    Connected,
}
