pub mod ratpad_communication {
    use serde::{Deserialize, Serialize};
    use serde_json::{Error, Value};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Message {
        pub header: String,
        pub data: Option<Value>,
    }

    pub fn create_message(header: String, data: Option<Value>) -> Result<String, Error> {
        if let Some(confirmed_data) = data {
            match serde_json::to_string(&confirmed_data) {
                Ok(ser) => Ok(format!("{header}:{ser};\n")),
                Err(err) => Err(err),
            }
        } else {
            Ok(format!("{header}:;\n"))
        }
    }

    pub fn create_empty_message(header: String) -> Result<String, Error> {
        create_message(header, None)
    }

    pub fn parse_message(msg: String) -> Option<Message> {
        if let Some((header, data)) = msg.trim().trim_end_matches(';').split_once(":") {
            if data.len() > 0 {
                match serde_json::from_str(data) {
                    Ok(parsed) => Some(Message {header: header.to_string(), data: parsed}),
                    Err(_) => None
                }
            } else {
                Some(Message {header: header.to_string(), data: None})
            }
        } else {
            None
        }
    }

    #[derive(Serialize,Deserialize, Clone, Debug)]
    pub struct ColorsConfig {
        next: (u32, u32, u32),
        previous: (u32, u32, u32),
        select: (u32, u32, u32),
        brightness: f64
    }

    #[derive(Serialize,Deserialize, Clone, Debug)]
    pub enum ColorKey {
        Next,
        Previous,
        Select,
        Brightness
    }

    #[derive(Serialize,Deserialize, Clone, Debug)]
    pub struct ModeKey {
        label: String,
        keys: String,
        color: Option<(u32, u32, u32)>
    }

    #[derive(Serialize,Deserialize, Clone, Debug)]
    pub struct ModeConfig {
        key: String,
        title: String,
        title_short: String,
        keys: Vec<Option<ModeKey>>,
        color: Option<(u32, u32, u32)>
    }

    #[derive(Serialize,Deserialize, Clone, Debug)]
    pub struct PadConfig {
        colors: ColorsConfig,
        modes: Vec<ModeConfig>
    }

    pub mod messages {
        use serde::{Deserialize, Serialize};

        use super::{ColorKey, ModeConfig, PadConfig};

        #[derive(Serialize,Deserialize, Clone, Debug)]
        pub struct KeyInfo {
            code: u32,
            name: String,
            action: u32
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub enum PadMessage {
            KeyEvent{mode: String, key: KeyInfo},
            EncoderSwitchEvent{mode: String, pressed: bool},
            EncoderRotationEvent{mode: String, value: i64},
            Log{content: String, level: String},
            Connect,
            Disconnect,
            Config{config: PadConfig}
        }

        #[derive(Clone, Debug, Serialize, Deserialize)]
        pub enum PadCommand {
            SetColor{key: ColorKey, color: (u32, u32, u32)},
            WriteMode{mode: ModeConfig},
            DeleteMode{key: String},
            ClearModes,
            ReadConfig
        }
    }
}
