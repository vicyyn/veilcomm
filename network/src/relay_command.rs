pub enum RelayCommand {
    Begin = 1,
    Data = 2,
    End = 3,
    Connected = 4,
    SendMe = 5,
    Extend = 6,
    Extended = 7,
    Truncate = 8,
    Truncated = 9,
    Drop = 10,
    Resolve = 11,
    Resolved = 12,
    BeginDir = 13,
    Extend2 = 14,
    Extended2 = 15,
    EstablishIntro = 32,
    EstablishRendPoint = 33,
    Introduce1 = 34,
    IntroEstablished = 38,
    RendPointEstablished = 39,
}

impl TryFrom<u8> for RelayCommand {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Data),
            3 => Ok(Self::End),
            4 => Ok(Self::Connected),
            5 => Ok(Self::SendMe),
            6 => Ok(Self::Extend),
            7 => Ok(Self::Extended),
            8 => Ok(Self::Truncate),
            9 => Ok(Self::Truncated),
            10 => Ok(Self::Drop),
            11 => Ok(Self::Resolve),
            12 => Ok(Self::Resolved),
            13 => Ok(Self::BeginDir),
            14 => Ok(Self::Extend),
            15 => Ok(Self::Extend2),
            32 => Ok(Self::EstablishIntro),
            33 => Ok(Self::EstablishRendPoint),
            34 => Ok(Self::Introduce1),
            38 => Ok(Self::IntroEstablished),
            39 => Ok(Self::RendPointEstablished),
            _ => Err("unrecognized command"),
        }
    }
}
