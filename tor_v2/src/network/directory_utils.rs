use directory::{
    GetRelaysOutput, GetUsersOutput, PublishRelayInput, PublishUserInput, RelayDescriptor,
    UserDescriptor, DIRECTORY_ADDRESS,
};
use reqwest::blocking::Client;

fn get_config() -> (Client, String) {
    let client = Client::new();
    let url = format!("http://{}", DIRECTORY_ADDRESS);
    (client, url)
}

pub fn publish_user(user_descriptor: UserDescriptor) {
    let (client, url) = get_config();
    let res = client
        .post(url + "/publish_user")
        .json(&PublishUserInput { user_descriptor })
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);
}

pub fn get_users() -> Vec<UserDescriptor> {
    let (client, url) = get_config();
    let get_users_output = client
        .get(url + "/get_users")
        .send()
        .unwrap()
        .json::<GetUsersOutput>()
        .unwrap();
    return get_users_output.users;
}

pub fn publish_relay(relay_descriptor: RelayDescriptor) {
    let (client, url) = get_config();
    let res = client
        .post(url + "/publish_relay")
        .json(&PublishRelayInput { relay_descriptor })
        .send()
        .unwrap();
    assert_eq!(res.status(), 200);
}

pub fn get_relays() -> Vec<RelayDescriptor> {
    let (client, url) = get_config();
    let get_relays_output = client
        .get(url + "/get_relays")
        .send()
        .unwrap()
        .json::<GetRelaysOutput>()
        .unwrap();

    return get_relays_output.relays;
}
