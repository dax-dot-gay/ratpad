// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;


use tauri::Manager;
use util::ratpad_communication::Message;
pub use util::serial_client;
pub use util::ratpad_communication;
use util::serial_client::{get_ports, PortInfo, SerialError, start_serial_listener, ListenerCommand};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn list_ports() -> Result<Vec<PortInfo>, SerialError> {
    let result = get_ports();
    match result {
        Ok(val) => Ok(val),
        Err(error_data) => Err(error_data)
    }
}

#[tauri::command]
async fn connect(app: tauri::AppHandle, port: String, rate: u32) -> Result<(), String> {
    if let Ok(data) = serde_json::to_string(&ListenerCommand::Connect { new_port: port.clone(), new_rate: rate }) {
        app.trigger_global("ratpad://serial/cmd", Some(data));
    }
    Ok(())
}

#[tauri::command]
async fn send_serial(app: tauri::AppHandle, message: Message) -> Result<(), String> {
    if let Ok(data) = serde_json::to_string(&ListenerCommand::Send(message)) {
        app.trigger_global("ratpad://serial/cmd", Some(data));
    }
    Ok(())
}

#[tauri::command]
async fn disconnect(app: tauri::AppHandle) -> Result<(), String> {
    if let Ok(data) = serde_json::to_string(&ListenerCommand::Disconnect) {
        app.trigger_global("ratpad://serial/cmd", Some(data));
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            start_serial_listener(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_ports, connect, send_serial, disconnect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
