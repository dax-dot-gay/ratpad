pub mod ratpad_communication {
    use serde::{Deserialize, Serialize};
    use serde_json::{Error, Value};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum EventType {
        Event,
        Connect,
        Disconnect,
        Log,
        Config,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum CommandType {
        SetColor,
        WriteMode,
        DeleteMode,
        ClearModes,
        ReadConfig,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum MessageType {
        Event(EventType),
        Command(CommandType),
        Unknown,
    }

    impl Serialize for MessageType {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for MessageType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let val: &str = Deserialize::deserialize(deserializer)?;
            Ok(MessageType::from_str(val))
        }
    }

    impl MessageType {
        fn as_str(&self) -> &'static str {
            match self {
                MessageType::Event(subtype) => match subtype {
                    EventType::Config => "config",
                    EventType::Connect => "connect",
                    EventType::Disconnect => "disconnect",
                    EventType::Event => "event",
                    EventType::Log => "log",
                },
                MessageType::Command(subtype) => match subtype {
                    CommandType::ClearModes => "clear_modes",
                    CommandType::DeleteMode => "delete_mode",
                    CommandType::ReadConfig => "read_config",
                    CommandType::SetColor => "set_color",
                    CommandType::WriteMode => "write_mode",
                },
                MessageType::Unknown => "unknown",
            }
        }

        fn from_str(value: &str) -> MessageType {
            match value {
                "config" => MessageType::Event(EventType::Config),
                "connect" => MessageType::Event(EventType::Connect),
                "disconnect" => MessageType::Event(EventType::Disconnect),
                "event" => MessageType::Event(EventType::Event),
                "log" => MessageType::Event(EventType::Log),
                "clear_modes" => MessageType::Command(CommandType::ClearModes),
                "delete_mode" => MessageType::Command(CommandType::DeleteMode),
                "read_config" => MessageType::Command(CommandType::ReadConfig),
                "set_color" => MessageType::Command(CommandType::SetColor),
                "write_mode" => MessageType::Command(CommandType::WriteMode),
                _ => MessageType::Unknown,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Message {
        pub message_type: MessageType,
        pub data: Option<Value>,
    }

    pub fn create_message(message: Message) -> Result<String, Error> {
        let message_type = message.message_type.as_str();
        let data = message.data;
        if let Some(confirmed_data) = data {
            match serde_json::to_string(&confirmed_data) {
                Ok(ser) => Ok(format!("{message_type}:{ser};\n")),
                Err(err) => Err(err),
            }
        } else {
            Ok(format!("{message_type}:;\n"))
        }
    }

    pub fn create_empty_message(message_type: MessageType) -> Result<String, Error> {
        create_message(Message {message_type, data: None})
    }

    pub fn parse_message(msg: String) -> Option<Message> {
        if let Some((header, data)) = msg.trim().trim_end_matches(';').split_once(":") {
            if data.len() > 0 {
                match serde_json::from_str(data) {
                    Ok(parsed) => Some(Message {
                        message_type: MessageType::from_str(&*header.to_ascii_lowercase()),
                        data: parsed,
                    }),
                    Err(_) => None,
                }
            } else {
                Some(Message {
                    message_type: MessageType::from_str(&*header.to_ascii_lowercase()),
                    data: None,
                })
            }
        } else {
            None
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ColorsConfig {
        pub next: (u32, u32, u32),
        pub previous: (u32, u32, u32),
        pub select: (u32, u32, u32),
        pub brightness: f64,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub enum ColorKey {
        Next,
        Previous,
        Select,
        Brightness,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ModeKey {
        pub label: String,
        pub keys: Option<String>,
        pub color: Option<(u32, u32, u32)>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct ModeConfig {
        pub key: String,
        pub title: String,
        pub title_short: String,
        pub keys: Vec<Option<ModeKey>>,
        pub color: Option<(u32, u32, u32)>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct PadConfig {
        pub colors: ColorsConfig,
        pub modes: Vec<ModeConfig>,
    }
}
