pub mod command_handler {
    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Manager};

    use crate::serial_client::{get_ports, ListenerCommand, PortInfo};

    #[derive(Serialize, Deserialize, Clone)]
    #[serde(tag = "type")]
    pub enum CommandTypes {
        #[serde(rename = "serial.connect")]
        SerialConnect { port: String, rate: u32 },

        #[serde(rename = "serial.disconnect")]
        SerialDisconnect,

        #[serde(rename = "serial.list_ports")]
        SerialListPorts,
    }

    #[derive(Serialize, Deserialize, Clone)]
    #[serde(tag = "type")]
    pub enum CommandReturnTypes {
        #[serde(rename = "serial.connect")]
        SerialConnect {},

        #[serde(rename = "serial.disconnect")]
        SerialDisconnect {},

        #[serde(rename = "serial.list_ports")]
        SerialListPorts { result: Vec<PortInfo> },
    }

    pub fn execute(app: AppHandle, command: CommandTypes) -> Result<CommandReturnTypes, &'static str> {
        match command {
            CommandTypes::SerialConnect { port, rate } => {
                serde_json::to_string(&ListenerCommand::Connect {
                    new_port: port,
                    new_rate: rate,
                })
                .or(Err("Command serialization failed"))
                .and_then(|com| {app.trigger_global("ratpad://serial", Some(com)); Ok(CommandReturnTypes::SerialConnect{})})
            },
            CommandTypes::SerialDisconnect => {
                serde_json::to_string(&ListenerCommand::Disconnect)
                .or(Err("Command serialization failed"))
                .and_then(|com| {app.trigger_global("ratpad://serial", Some(com)); Ok(CommandReturnTypes::SerialDisconnect{})})
            },
            CommandTypes::SerialListPorts => get_ports().or(Err("Port list failure")).and_then(|res| Ok(CommandReturnTypes::SerialListPorts{result: res}))
        }
    }
}
