use crate::relay::RelayDescriptor;
use crate::user::UserDescriptor;
use crate::{IntroductionPointId, Logger, RelayId};
use anyhow::Result;
use lazy_static::lazy_static;
use std::sync::Mutex;
use uuid::Uuid;

lazy_static! {
    pub static ref directory: Directory = Directory {
        relays: Mutex::new(Vec::new()),
        users: Mutex::new(Vec::new()),
    };
}

pub struct Directory {
    relays: Mutex<Vec<RelayDescriptor>>,
    users: Mutex<Vec<UserDescriptor>>,
}

impl Directory {
    pub fn get_relays() -> Vec<RelayDescriptor> {
        Logger::info("Directory", "Fetching all relays");
        directory.relays.lock().unwrap().clone()
    }

    pub fn get_users() -> Vec<UserDescriptor> {
        Logger::info("Directory", "Fetching all users");
        directory.users.lock().unwrap().clone()
    }

    pub fn get_relay(relay_id: RelayId) -> Option<RelayDescriptor> {
        Logger::info("Directory", format!("Fetching relay {}", relay_id));
        let relays = directory.relays.lock().unwrap();
        relays.iter().find(|r| r.id == relay_id).cloned()
    }

    pub fn get_user(user_id: Uuid) -> Option<UserDescriptor> {
        Logger::info("Directory", format!("Fetching user {}", user_id));
        let users = directory.users.lock().unwrap();
        users.iter().find(|u| u.id == user_id).cloned()
    }

    pub fn publish_relay(relay: RelayDescriptor) {
        Logger::info(
            "Directory",
            format!("Publishing a new relay {}", relay.nickname),
        );
        let mut relays = directory.relays.lock().unwrap();
        relays.push(relay);
    }

    pub fn publish_user(user: UserDescriptor) {
        Logger::info(
            "Directory",
            format!("Publishing a new user {}", user.nickname),
        );
        let mut users = directory.users.lock().unwrap();
        users.push(user);
    }

    pub fn add_user_introduction_point(
        user_id: Uuid,
        introduction_points: IntroductionPointId,
        relay_id: RelayId,
    ) -> Result<()> {
        Logger::info(
            "Directory",
            format!("Adding introduction point for user {}", user_id),
        );
        let mut users = directory.users.lock().unwrap();
        let user = users
            .iter_mut()
            .find(|u| u.id == user_id)
            .ok_or(anyhow::anyhow!("User not found"))?;
        user.introduction_points
            .insert(introduction_points, relay_id);
        Ok(())
    }
}
