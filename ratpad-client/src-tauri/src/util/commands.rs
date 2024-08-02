pub mod command_handler {
    use std::{sync::mpsc, time::Duration};

    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Manager};

    use crate::{
        ratpad_communication::{CommandType, EventType, Message, MessageType, PadConfig},
        serial_client::{get_ports, send_serial_command, ListenerCommand, PortInfo, SerialEvent},
        util::app_state::{ApplicationState, ConnectionState},
    };

    #[derive(Serialize, Deserialize, Clone)]
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

        send_serial_command(
            app.clone(),
            command,
        );

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
            CommandTypes::PadGetConfig => {
                publish_and_wait(app.clone(),Message {
                        message_type: MessageType::Command(CommandType::ReadConfig),
                        data: None,
                    },  MessageType::Event(EventType::Config), None)
                    .or(Err("Failed to retrieve config"))
                    .and_then(|res| {
                        res.data
                            .ok_or(())
                            .or(Err("Config not returned by pad."))
                            .and_then(|data| {
                                serde_json::from_value::<PadConfig>(data)
                                    .or(Err("Config failed to parse"))
                                    .and_then(|p| {
                                        Ok(CommandReturnTypes::PadGetConfig { config: p })
                                    })
                            })
                    })
            }
        }
    }
}
