use super::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Payload {
    EstablishRendezvous(EstablishRendezvousPayload),
    EstablishedRendezvous(EstablishedRendezvousPayload),
    EstablishIntroduction(EstablishIntroductionPayload),
    EstablishedIntroduction(EstablishedIntroductionPayload),
    Create(CreatePayload),
    Created(CreatedPayload),
    Extend(ExtendPayload),
    Extended(ExtendedPayload),
    Begin(BeginPayload),
    Connected(ConnectedPayload),
    Introduce1(Introduce1Payload),
    Introduce2(Introduce2Payload),
    IntroduceAck(IntroduceAckPayload),
    Rendezvous1(Rendezvous1Payload),
    Rendezvous2(Rendezvous2Payload),
    Data(DataPayload),
}

#[derive(PartialEq, Eq, Debug)]
pub enum PayloadType {
    Create,
    Created,
    Extend,
    Extended,
    EstablishRendezvous,
    EstablishedRendezvous,
    EstablishIntroduction,
    EstablishedIntroduction,
    Begin,
    Connected,
    Introduce1,
    Introduce2,
    IntroduceAck,
    Rendezvous1,
    Rendezvous2,
    Data,
}

impl Payload {
    pub fn get_type(&self) -> PayloadType {
        match self {
            Payload::Create(_) => PayloadType::Create,
            Payload::Created(_) => PayloadType::Created,
            Payload::Extend(_) => PayloadType::Extend,
            Payload::Extended(_) => PayloadType::Extended,
            Payload::EstablishRendezvous(_) => PayloadType::EstablishRendezvous,
            Payload::EstablishedRendezvous(_) => PayloadType::EstablishedRendezvous,
            Payload::EstablishIntroduction(_) => PayloadType::EstablishIntroduction,
            Payload::EstablishedIntroduction(_) => PayloadType::EstablishedIntroduction,
            Payload::Begin(_) => PayloadType::Begin,
            Payload::Connected(_) => PayloadType::Connected,
            Payload::Introduce1(_) => PayloadType::Introduce1,
            Payload::Introduce2(_) => PayloadType::Introduce2,
            Payload::IntroduceAck(_) => PayloadType::IntroduceAck,
            Payload::Rendezvous1(_) => PayloadType::Rendezvous1,
            Payload::Rendezvous2(_) => PayloadType::Rendezvous2,
            Payload::Data(_) => PayloadType::Data,
        }
    }
}
