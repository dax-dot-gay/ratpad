pub mod ratpad_communication {
    use serde::{Deserialize, Serialize};
    use serde_json::{Error, Value};

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Message {
        header: String,
        data: Option<Value>,
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
}
