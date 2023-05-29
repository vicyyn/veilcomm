// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::mpsc::Receiver,
    thread,
};

use rand::Rng;
use tauri::{AppHandle, Manager};
use tor_v2::{start_peer, tor_change, TorEvent};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, serde::Serialize)]
pub struct TorEventPayload {
    event: String,
    message: String,
    port: u16,
}

#[derive(Clone, serde::Serialize)]
pub struct TorChangeFetchRelay {
    ip: String,
    port: u16,
    id_key: String,
}

fn start_tor_change_listener(
    tor_change_reader: Receiver<tor_change::TorChange>,
    app_handle: AppHandle,
) {
    thread::spawn(move || loop {
        let tor_change = tor_change_reader.recv().unwrap();
        match tor_change {
            tor_change::TorChange::ReceiveMessage(_) => todo!(),
            tor_change::TorChange::SendMessage(_) => todo!(),
            tor_change::TorChange::Logs(logs) => {
                app_handle
                    .emit_all::<String>("tor-change-logs", logs)
                    .unwrap();
            }
            tor_change::TorChange::ReceiveRelays(relays) => {
                let mut changes = vec![];
                for relay in relays {
                    changes.push(TorChangeFetchRelay {
                        ip: relay.socket_address.ip().to_string(),
                        port: relay.socket_address.port(),
                        id_key: hex::encode(&relay.identity_key),
                    })
                }
                app_handle
                    .emit_all::<Vec<TorChangeFetchRelay>>("tor-change-fetch-relays", changes)
                    .unwrap();
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // start peer
            let mut rng = rand::thread_rng();
            let port: u16 = rng.gen_range(1024..=65535);
            let (tor_event_sender, tor_change_reader) =
                start_peer(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
            start_tor_change_listener(tor_change_reader, app.handle());

            app.listen_global("tor-event", move |event| match event.payload() {
                Some(payload) => match &payload.to_lowercase() {
                    x if x.contains(&"fetch-relays") => {
                        tor_event_sender.send(TorEvent::FetchFromDirectory).unwrap();
                    }
                    x if x.contains(&"initialize") => {
                        if x.contains(&"true") {
                            tor_event_sender
                                .send(TorEvent::InitializePeer(true))
                                .unwrap();
                        } else {
                            tor_event_sender
                                .send(TorEvent::InitializePeer(false))
                                .unwrap();
                        }
                    }
                    _ => {}
                },
                _ => {}
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
