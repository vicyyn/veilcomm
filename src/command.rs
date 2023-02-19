#[repr(u8)]
pub enum Command {
    // The 'Command' field of a fixed-length cell holds one of the following
    Padding = 0,
    Create = 1,
    Created = 2,
    Relay = 3,
    Destroy = 4,
    CreateFast = 5,
    CreatedFast = 6,
    NetInfo = 8,
    RelayEarly = 9,
    Create2 = 10,
    Created2 = 11,
    PaddingNegotiate = 12,

    // Variable-length command values are:
    Versions = 7,
    VPadding = 128,
    Certs = 129,
    AuthChallenge = 130,
    Authenticate = 131,
    Authorize = 132,
}

impl TryFrom<u8> for Command {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Command::Padding),
            1 => Ok(Command::Create),
            2 => Ok(Command::Created),
            3 => Ok(Command::Relay),
            4 => Ok(Command::Destroy),
            5 => Ok(Command::CreateFast),
            6 => Ok(Command::CreatedFast),
            7 => Ok(Command::Versions),
            8 => Ok(Command::NetInfo),
            9 => Ok(Command::RelayEarly),
            10 => Ok(Command::Create2),
            11 => Ok(Command::Created2),
            12 => Ok(Command::PaddingNegotiate),
            128 => Ok(Command::VPadding),
            129 => Ok(Command::Certs),
            130 => Ok(Command::AuthChallenge),
            131 => Ok(Command::Authenticate),
            132 => Ok(Command::Authorize),
            _ => Err("unrecognized command"),
        }
    }
}

impl Command {}
