// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;
pub use util::serial_client;
use util::serial_client::{get_ports, SerialError, PortInfo};

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
        .invoke_handler(tauri::generate_handler![list_ports])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
