// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    thread,
};

use rand::Rng;
use tauri::Manager;
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // start peer
            let mut rng = rand::thread_rng();
            let port: u16 = rng.gen_range(1024..=65535);
            let (tor_event_sender, tor_change_reader) =
                start_peer(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port));

            app.listen_global("tor-event", |event| {
                println!("got tor-event with payload {:?}", event.payload());
            });

            let tor_change = tor_change_reader.recv().unwrap();
            match tor_change {
                tor_change::TorChange::ReceiveMessage(_) => todo!(),
                tor_change::TorChange::SendMessage(_) => todo!(),
                tor_change::TorChange::Logs(logs) => {
                    println!("{}", logs);
                    app.emit_all::<String>("tor-change", logs).unwrap();
                }
                tor_change::TorChange::ReceiveRelays(_) => todo!(),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
