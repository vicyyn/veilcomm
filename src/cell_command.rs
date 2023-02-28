use std::convert::TryFrom;

pub enum CellCommand {
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

impl TryFrom<u8> for CellCommand {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Padding),
            1 => Ok(Self::Create),
            2 => Ok(Self::Created),
            3 => Ok(Self::Relay),
            4 => Ok(Self::Destroy),
            5 => Ok(Self::CreateFast),
            6 => Ok(Self::CreatedFast),
            7 => Ok(Self::Versions),
            8 => Ok(Self::NetInfo),
            9 => Ok(Self::RelayEarly),
            10 => Ok(Self::Create2),
            11 => Ok(Self::Created2),
            12 => Ok(Self::PaddingNegotiate),
            128 => Ok(Self::VPadding),
            129 => Ok(Self::Certs),
            130 => Ok(Self::AuthChallenge),
            131 => Ok(Self::Authenticate),
            132 => Ok(Self::Authorize),
            _ => Err("unrecognized command"),
        }
    }
}
