pub mod configuration {
    use std::{fs::{create_dir_all, read_to_string, File}, io::Write};

    use serde::{Deserialize, Serialize};
    use serde_json::Error;
    use tauri::AppHandle;

    use crate::{ratpad_communication::{ColorsConfig, ModeConfig, ModeKey, PadConfig}, util::command_handler::SetColorType};

    pub type Color = (u32, u32, u32);

    pub trait PadCompat<I, O> {
        fn to_pad(&self) -> O;
        fn from_pad(value: O) -> I; 
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum KeyAction {
        #[serde(rename = "none")]
        None{},

        #[serde(rename = "keypress")]
        KeyPress{key: String},

        #[serde(rename = "command")]
        Command{
            execute: String,
            args: Option<Vec<String>>
        }
    }

    impl PadCompat<KeyAction, Option<String>> for KeyAction {
        fn to_pad(&self) -> Option<String> {
            match self {
                KeyAction::KeyPress{key} => Some(key.clone()),
                _ => None
            }
        }

        fn from_pad(value: Option<String>) -> KeyAction {
            if let Some(val) = value {
                KeyAction::KeyPress{key: val}
            } else {
                KeyAction::None{}
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct KeyConfig {
        label: String,
        action: KeyAction,
        color: Option<Color>
    }

    impl PadCompat<KeyConfig, ModeKey> for KeyConfig {
        fn to_pad(&self) -> ModeKey {
            ModeKey {
                label: self.label.clone(),
                keys: self.action.to_pad(),
                color: self.color
            }
        }

        fn from_pad(value: ModeKey) -> KeyConfig {
            KeyConfig {
                label: value.label,
                action: KeyAction::from_pad(value.keys),
                color: value.color
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AppModeConfig {
        key: String,
        title: String,
        title_short: String,
        color: Option<Color>,
        keys: Vec<Option<KeyConfig>>
    }

    impl PadCompat<AppModeConfig, ModeConfig> for AppModeConfig {
        fn to_pad(&self) -> ModeConfig {
            ModeConfig {
                key: self.key.clone(),
                title: self.title.clone(),
                title_short: self.title_short.clone(),
                keys: self.keys.iter().map(|v| {
                    if let Some(key) = v {
                        Some(key.to_pad())
                    } else {
                        None
                    }
                }).collect(),
                color: self.color
            }
        }

        fn from_pad(value: ModeConfig) -> AppModeConfig {
            AppModeConfig {
                key: value.key,
                title: value.title,
                title_short: value.title_short,
                color: value.color,
                keys: value.keys.iter().map(|v| {
                    if let Some(key) = v {
                        Some(KeyConfig::from_pad(key.clone()))
                    } else {
                        None
                    }
                }).collect()
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct AppConfig {
        pub device_port: Option<String>,
        pub device_rate: Option<u32>,
        pub colors: ColorsConfig,
        pub modes: Vec<AppModeConfig>
    }

    impl PadCompat<AppConfig, PadConfig> for AppConfig {
        fn to_pad(&self) -> PadConfig {
            PadConfig {
                colors: self.colors.clone(),
                modes: self.modes.iter().map(|v| v.to_pad()).collect()
            }
        }

        fn from_pad(value: PadConfig) -> AppConfig {
            AppConfig {
                device_port: None,
                device_rate: None,
                colors: value.colors.clone(),
                modes: value.modes.iter().map(|v| AppModeConfig::from_pad(v.clone())).collect()

            }
        }
    }

    impl AppConfig {
        pub fn default() -> AppConfig {
            AppConfig {
                device_port: None,
                device_rate: None,
                colors: ColorsConfig { next: (0, 0, 0), previous: (0, 0, 0), select: (0, 0, 0), brightness: 1.0 },
                modes: Vec::new()
            }
        }

        pub fn from_json(value: &str) -> Result<AppConfig, Error> {
            serde_json::from_str::<AppConfig>(value)
        }

        pub fn to_json(&self) -> Result<String, Error> {
            serde_json::to_string(self)
        }

        pub fn load_config(app: AppHandle) -> AppConfig {
            let mut path = app.path_resolver().app_config_dir().expect("Unable to resolve config path");
            path.push("config.json");
            if path.exists() {
                let data = read_to_string(path).expect("Unable to read config file (already exists)");
                AppConfig::from_json(data.as_str()).expect("Unable to parse config file.")
            } else {
                AppConfig::default().save(app)
            }
        }

        pub fn save(&self, app: AppHandle) -> AppConfig {
            let mut path = app.path_resolver().app_config_dir().expect("Unable to resolve config path");
            if !path.exists() {
                create_dir_all(path.clone()).expect("Unable to create config directory");
            }
            path.push("config.json");
            let mut file = File::create(path).expect("Unable to open config file for writing.");
            file.write_all(self.to_json().expect("Unable to serialize config").as_bytes()).expect("Unable to write data");
            self.clone()
        }

        pub fn set_connection(&mut self, port: String, rate: u32) -> AppConfig {
            self.device_port = Some(port);
            self.device_rate = Some(rate);
            self.clone()
        }

        pub fn clear_connection(&mut self) -> AppConfig {
            self.device_port = None;
            self.device_rate = None;
            self.clone()
        }

        pub fn set(&mut self, update: AppConfig) -> AppConfig {
            self.device_port = update.device_port;
            self.device_rate = update.device_rate;
            self.colors = update.colors;
            self.modes = update.modes;
            self.clone()
        }

        pub fn set_color(&mut self, color: SetColorType) -> AppConfig {
            match color {
                SetColorType::Next { color } => self.colors.next = color,
                SetColorType::Previous { color } => self.colors.previous = color,
                SetColorType::Select { color } => self.colors.select = color,
                SetColorType::Brightness { color } => self.colors.brightness = color,
            }
            self.clone()
        }

        pub fn write_mode(&mut self, mode: AppModeConfig) -> AppConfig {
            self.modes = self.modes.iter().filter_map(|m| {
                if m.key == mode.key {
                    None
                } else {
                    Some(m.clone())
                }
            }).collect();
            self.modes.push(mode);
            self.clone()
        }

        pub fn delete_mode(&mut self, mode: String) -> AppConfig {
            self.modes = self.modes.iter().filter_map(|m| {
                if m.key == mode {
                    None
                } else {
                    Some(m.clone())
                }
            }).collect();
            self.clone()
        }

        pub fn clear_modes(&mut self) -> AppConfig {
            self.modes.clear();
            self.clone()
        }
    }
}