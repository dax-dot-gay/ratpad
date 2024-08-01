// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;

use std::sync::Mutex;
use tauri::AppHandle;
use tauri::CustomMenuItem;
use tauri::SystemTray;

use tauri::Manager;
use tauri::SystemTrayEvent;
use tauri::SystemTrayMenu;
use tauri::SystemTrayMenuItem;
use tauri::WindowEvent;
use util::app_state::ApplicationState;
use util::app_state::ConnectionState;
use util::command_handler::execute;
use util::command_handler::CommandReturnTypes;
use util::command_handler::CommandTypes;
use util::configuration::AppConfig;
pub use util::ratpad_communication;
pub use util::serial_client;
use util::serial_client::SerialEvent;
use util::serial_client::{
    start_serial_listener, ListenerCommand
};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn execute_command(app: AppHandle, command: CommandTypes) -> Result<CommandReturnTypes, String> {
  match execute(app, command) {
    Ok(res) => Ok(res),
    Err(reason) => Err(reason.to_string())
  }
}

fn generate_tray() -> SystemTray {
    let title = CustomMenuItem::new("title".to_string(), "Ratpad Client").disabled();
    let status = CustomMenuItem::new("status".to_string(), "Status: Disconnected").disabled();
    let toggle = CustomMenuItem::new("toggle".to_string(), "Toggle Window");
    let menu = SystemTrayMenu::new()
        .add_item(title)
        .add_item(status)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(toggle);

    SystemTray::new().with_menu(menu)
}

fn main() {
    tauri::Builder::default()
        .manage(ApplicationState {
            connection: Mutex::new(ConnectionState::Disconnected),
            port: Mutex::new(None),
            rate: Mutex::new(None),
            config: Mutex::new(AppConfig::default()),
        })
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            app.trigger_global("ratpad://single-instance", None);
        }))
        .setup(|app| {
            (*app.state::<ApplicationState>().lock_config().unwrap())
                .set(AppConfig::load_config(app.handle()))
                .save(app.handle());
            start_serial_listener(app);
            let handle = app.handle();
            let evt_handle = handle.clone();
            app.listen_global("ratpad://serial", move |event| {
                if let Some(payload) = event.payload() {
                    if let Ok(parsed) = serde_json::from_str::<SerialEvent>(payload) {
                        evt_handle
                            .emit_all("ratpad://serial", parsed.clone())
                            .expect("Serial emit failed");

                        match parsed {
                            SerialEvent::Connect => evt_handle
                                .tray_handle()
                                .get_item("status")
                                .set_title("Status: Connected")
                                .unwrap(),
                            SerialEvent::Disconnect => evt_handle
                                .tray_handle()
                                .get_item("status")
                                .set_title("Status: Disconnected")
                                .unwrap(),
                            SerialEvent::Event(_) => (),
                        }
                    }
                }
            });
            let single_instance_handle = handle.clone();
            app.listen_global("ratpad://single-instance", move |_| {
                single_instance_handle
                    .get_window("main")
                    .unwrap()
                    .show()
                    .unwrap();
            });
            if let Some(config) = app.state::<ApplicationState>().lock_config() {
                if let Some(dev_port) = config.clone().device_port {
                    if let Some(dev_rate) = config.clone().device_rate {
                        if let Ok(data) = serde_json::to_string(&ListenerCommand::Connect {
                            new_port: dev_port.clone(),
                            new_rate: dev_rate,
                        }) {
                            app.trigger_global("ratpad://serial/cmd", Some(data));
                        }
                    }
                }
            }

            Ok(())
        })
        .system_tray(generate_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "toggle" => {
                    if let Some(window) = app.get_window("main") {
                        if let Ok(visible) = window.is_visible() {
                            if visible {
                                window.hide().unwrap();
                            } else {
                                window.show().unwrap();
                            }
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => (),
        })
        .invoke_handler(tauri::generate_handler![
            execute_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
