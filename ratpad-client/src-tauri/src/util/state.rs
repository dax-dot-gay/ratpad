pub mod app_state {
    use std::sync::{Mutex, MutexGuard};

    use serde::{Deserialize, Serialize};

    use crate::util::configuration::AppConfig;

    #[derive(Serialize, Deserialize, Clone)]
    pub enum ConnectionState {
        Connected,
        Waiting,
        Disconnected
    }

    pub struct ApplicationState {
        pub connection: Mutex<ConnectionState>,
        pub port: Mutex<Option<String>>,
        pub rate: Mutex<Option<u32>>,
        pub config: Mutex<AppConfig>
    }

    impl ApplicationState {
        pub fn set(&self, connection: ConnectionState, new_port: Option<String>, new_rate: Option<u32>) -> () {
            if let Ok(mut conn) = self.connection.lock() {
                *conn = connection;
            }

            if let Ok(mut port) = self.port.lock() {
                *port = new_port;
            }

            if let Ok(mut rate) = self.rate.lock() {
                *rate = new_rate;
            }
        }

        pub fn set_connection_state(&self, connection: ConnectionState) -> () {
            if let Ok(mut conn) = self.connection.lock() {
                *conn = connection;
            }
        }

        pub fn set_connection_parameters(&self, new_port: Option<String>, new_rate: Option<u32>) -> () {
            if let Ok(mut port) = self.port.lock() {
                *port = new_port;
            }

            if let Ok(mut rate) = self.rate.lock() {
                *rate = new_rate;
            }
        }

        pub fn lock_config(&self) -> Option<MutexGuard<AppConfig>> {
            if let Ok(conf) = self.config.lock() {
                Some(conf)
            } else {
                None
            }
        }
    }
}