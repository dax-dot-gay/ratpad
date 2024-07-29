// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;


pub use util::serial_client;
pub use util::ratpad_communication;
use util::serial_client::{get_ports, listen_serial, PortInfo, SerialError};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn list_ports() -> Result<Vec<PortInfo>, SerialError> {
    let result = get_ports();
    match result {
        Ok(val) => Ok(val),
        Err(error_data) => Err(error_data)
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            listen_serial(handle, "/dev/ttyACM1", 115200).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_ports])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
