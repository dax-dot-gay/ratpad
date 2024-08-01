pub mod app_state {
    use std::sync::Mutex;

    use crate::util::configuration::AppConfig;

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

        pub fn set_config(&self, new_config: Option<AppConfig>) -> () {
            if let Ok(mut config) = self.config.lock() {
                *config = new_config.unwrap_or(AppConfig::default());
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
    }
}