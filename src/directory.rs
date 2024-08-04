use crate::relay::RelayDescriptor;
use crate::user::UserDescriptor;
use crate::Logger;
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

    pub fn update_user_introduction_points(user_id: Uuid, introduction_points: Vec<(Uuid, Uuid)>) {
        Logger::info(
            "Directory",
            format!("Updating introduction points for user {}", user_id),
        );
        let mut users = directory.users.lock().unwrap();
        for user in users.iter_mut() {
            if user.id == user_id {
                user.introduction_points = introduction_points;
                return;
            }
        }
    }
}
