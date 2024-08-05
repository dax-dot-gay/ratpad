pub mod command_handler {
    use std::{sync::mpsc, time::Duration};

    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tauri::{AppHandle, Manager};

    use crate::{
        ratpad_communication::{
            CommandType, EventType, Message, MessageType, ModeConfig, PadConfig,
        },
        serial_client::{get_ports, send_serial_command, ListenerCommand, PortInfo, SerialEvent},
        util::{
            app_state::{ApplicationState, ConnectionState},
            configuration::{AppConfig, AppModeConfig, PadCompat},
        },
    };

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "key")]
    pub enum SetColorType {
        #[serde(rename = "next")]
        Next { color: (u32, u32, u32) },

        #[serde(rename = "previous")]
        Previous { color: (u32, u32, u32) },

        #[serde(rename = "select")]
        Select { color: (u32, u32, u32) },

        #[serde(rename = "brightness")]
        Brightness { color: f64 },
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum CommandTypes {
        #[serde(rename = "serial.connect")]
        SerialConnect { port: String, rate: u32 },

        #[serde(rename = "serial.disconnect")]
        SerialDisconnect,

        #[serde(rename = "serial.list_ports")]
        SerialListPorts,

        #[serde(rename = "serial.get_state")]
        SerialGetState,

        #[serde(rename = "config.get_config")]
        ConfGetConfig,

        #[serde(rename = "config.set_color")]
        ConfSetColor { color: SetColorType },

        #[serde(rename = "config.write_mode")]
        ConfWriteMode { mode: AppModeConfig },

        #[serde(rename = "config.delete_mode")]
        ConfDeleteMode { key: String },

        #[serde(rename = "config.clear_modes")]
        ConfClearModes,

        #[serde(rename = "pad.set_mode")]
        PadSetMode { mode: String },

        #[serde(rename = "pad.set_home")]
        PadSetHome,

        #[serde(rename = "pad.get_config")]
        PadGetConfig,
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

        #[serde(rename = "serial.get_state")]
        SerialGetState {
            connected: ConnectionState,
            port: Option<String>,
            rate: Option<u32>,
        },

        #[serde(rename = "config.get_config")]
        ConfGetConfig { config: AppConfig },

        #[serde(rename = "config.set_color")]
        ConfSetColor {},

        #[serde(rename = "config.write_mode")]
        ConfWriteMode {},

        #[serde(rename = "config.delete_mode")]
        ConfDeleteMode {},

        #[serde(rename = "config.clear_modes")]
        ConfClearModes {},

        #[serde(rename = "pad.set_mode")]
        PadSetMode {},

        #[serde(rename = "pad.set_home")]
        PadSetHome {},

        #[serde(rename = "pad.get_config")]
        PadGetConfig { config: PadConfig },
    }

    fn publish_and_wait(
        app: AppHandle,
        command: Message,
        message: MessageType,
        timeout: Option<Duration>,
    ) -> Result<Message, String> {
        let (tx, rx) = mpsc::channel::<Message>();
        let listener = app.listen_global("ratpad://serial", move |evt| {
            if let Some(payload) = evt.payload() {
                if let Ok(parsed) = serde_json::from_str::<SerialEvent>(payload) {
                    match parsed {
                        SerialEvent::Event(event) => {
                            if event.message_type == message {
                                tx.send(event).expect("Failed to send over mpsc");
                            }
                        }
                        _ => (),
                    }
                }
            }
        });

        send_serial_command(app.clone(), command);

        let result: Result<Message, String>;

        if let Some(to) = timeout {
            if let Ok(recv) = rx.recv_timeout(to) {
                result = Ok(recv);
            } else {
                result = Err("Timed out".to_string());
            }
        } else {
            if let Ok(recv) = rx.recv() {
                result = Ok(recv);
            } else {
                result = Err("RECV failed".to_string());
            }
        }

        app.unlisten(listener);
        return result;
    }

    pub fn execute(
        app: AppHandle,
        command: CommandTypes,
    ) -> Result<CommandReturnTypes, &'static str> {
        match command {
            CommandTypes::SerialConnect { port, rate } => {
                serde_json::to_string(&ListenerCommand::Connect {
                    new_port: port,
                    new_rate: rate,
                })
                .or(Err("Command serialization failed"))
                .and_then(|com| {
                    app.trigger_global("ratpad://serial", Some(com));
                    Ok(CommandReturnTypes::SerialConnect {})
                })
            }
            CommandTypes::SerialDisconnect => serde_json::to_string(&ListenerCommand::Disconnect)
                .or(Err("Command serialization failed"))
                .and_then(|com| {
                    app.trigger_global("ratpad://serial", Some(com));
                    Ok(CommandReturnTypes::SerialDisconnect {})
                }),
            CommandTypes::SerialListPorts => get_ports()
                .or(Err("Port list failure"))
                .and_then(|res| Ok(CommandReturnTypes::SerialListPorts { result: res })),
            CommandTypes::SerialGetState => Ok(CommandReturnTypes::SerialGetState {
                connected: app
                    .state::<ApplicationState>()
                    .connection
                    .lock()
                    .unwrap()
                    .clone(),
                port: app.state::<ApplicationState>().port.lock().unwrap().clone(),
                rate: *app.state::<ApplicationState>().rate.lock().unwrap(),
            }),
            CommandTypes::PadGetConfig => publish_and_wait(
                app.clone(),
                Message {
                    message_type: MessageType::Command(CommandType::ReadConfig),
                    data: None,
                },
                MessageType::Event(EventType::Config),
                None,
            )
            .or(Err("Failed to retrieve config"))
            .and_then(|res| {
                res.data
                    .ok_or(())
                    .or(Err("Config not returned by pad."))
                    .and_then(|data| {
                        serde_json::from_value::<PadConfig>(data)
                            .or(Err("Config failed to parse"))
                            .and_then(|p| Ok(CommandReturnTypes::PadGetConfig { config: p }))
                    })
            }),
            CommandTypes::ConfSetColor { color } => {
                if let Ok(parsed) = serde_json::to_value::<SetColorType>(color.clone()) {
                    if let Some(mut state) = app.clone().state::<ApplicationState>().lock_config() {
                        state.set_color(color.clone()).save(app.clone());
                    } else {
                        return Err("Failed to lock state");
                    }
                    send_serial_command(
                        app.clone(),
                        Message {
                            message_type: MessageType::Command(CommandType::SetColor),
                            data: Some(parsed),
                        },
                    );
                    Ok(CommandReturnTypes::ConfSetColor {})
                } else {
                    Err("Failed to parse color data")
                }
            }
            CommandTypes::ConfWriteMode { mode } => {
                if let Ok(parsed) = serde_json::to_value::<ModeConfig>(mode.clone().to_pad()) {
                    if let Some(mut state) = app.clone().state::<ApplicationState>().lock_config() {
                        state.write_mode(mode).save(app.clone());
                    } else {
                        return Err("Failed to lock state");
                    }
                    send_serial_command(
                        app.clone(),
                        Message {
                            message_type: MessageType::Command(CommandType::WriteMode),
                            data: Some(parsed),
                        },
                    );
                    Ok(CommandReturnTypes::ConfWriteMode {})
                } else {
                    Err("Failed to parse mode data")
                }
            }
            CommandTypes::ConfDeleteMode { key } => {
                if let Some(mut state) = app.clone().state::<ApplicationState>().lock_config() {
                    state.delete_mode(key.clone()).save(app.clone());
                } else {
                    return Err("Failed to lock state");
                }
                send_serial_command(
                    app.clone(),
                    Message {
                        message_type: MessageType::Command(CommandType::DeleteMode),
                        data: Some(json!({"key": key})),
                    },
                );
                Ok(CommandReturnTypes::ConfDeleteMode {})
            }
            CommandTypes::ConfClearModes => {
                if let Some(mut state) = app.clone().state::<ApplicationState>().lock_config() {
                    state.clear_modes().save(app.clone());
                } else {
                    return Err("Failed to lock state");
                }
                send_serial_command(
                    app.clone(),
                    Message {
                        message_type: MessageType::Command(CommandType::ClearModes),
                        data: None,
                    },
                );
                Ok(CommandReturnTypes::ConfClearModes {})
            }
            CommandTypes::PadSetHome => {
                send_serial_command(
                    app.clone(),
                    Message {
                        message_type: MessageType::Command(CommandType::SetHome),
                        data: None,
                    },
                );
                Ok(CommandReturnTypes::PadSetHome {})
            }
            CommandTypes::PadSetMode { mode } => {
                send_serial_command(
                    app.clone(),
                    Message {
                        message_type: MessageType::Command(CommandType::SetMode),
                        data: Some(json!({"mode": mode})),
                    },
                );
                Ok(CommandReturnTypes::PadSetMode {})
            }
            CommandTypes::ConfGetConfig => {
                Ok(CommandReturnTypes::ConfGetConfig {
                    config: AppConfig::load_config(app.clone()),
                })
            }
        }
    }
}
