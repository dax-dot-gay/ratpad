pub mod serial_client {
    use std::{error::Error, fmt, io::{BufRead, BufReader, ErrorKind}, thread::{self, JoinHandle}};

    use serde::Serialize;
    use tauri::{AppHandle, Manager};
    use tokio_serial::{available_ports, SerialPortType, UsbPortInfo};

    #[derive(Serialize)]
    #[derive(Debug)]
    pub enum SerialErrorType {
        List,
        Connection(String)
    }

    #[derive(Serialize)]
    #[derive(Debug)]
    pub struct SerialError {
        error_type: SerialErrorType,
        error_source: String,
    }

    impl fmt::Display for SerialError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Serial error")
        }
    }

    impl Error for SerialError {}

    #[derive(Serialize)]
    #[serde(remote = "UsbPortInfo")]
    pub struct UsbPortInfoDef {
        pub vid: u16,
        pub pid: u16,
        pub serial_number: Option<String>,
        pub manufacturer: Option<String>,
        pub product: Option<String>,
    }

    #[derive(Serialize)]
    #[serde(remote = "SerialPortType")]
    pub enum SerialPortTypeDef {
        #[serde(with = "UsbPortInfoDef")]
        UsbPort(UsbPortInfo),

        PciPort,
        BluetoothPort,
        Unknown
    }

    #[derive(Serialize)]
    pub struct PortInfo {
        port_name: String,

        #[serde(with = "SerialPortTypeDef")]
        port_type: SerialPortType,
    }

    #[derive(Serialize, Clone)]
    pub enum SerialEvent {
        Event(String),
        Disconnect
    }

    pub fn get_ports() -> Result<Vec<PortInfo>, SerialError> {
        let result = available_ports();
        match result {
            Ok(ports) => Ok(ports.iter().map(|r| PortInfo {port_name: r.port_name.clone(), port_type: r.port_type.clone()} ).collect()),
            Err(_) => Err(SerialError { error_type: SerialErrorType::List, error_source: "get_ports".to_string() })
        }
    }

    pub fn listen_serial(handle: AppHandle, port: &str, baud_rate: u32) -> Result<JoinHandle<()>, SerialError> {
        let port_builder = tokio_serial::new(port, baud_rate);
        let opened_port = match port_builder.open() {
            Ok(port) => port,
            Err(_) => return Err(SerialError {error_type: SerialErrorType::Connection("conn_open".into()), error_source: "listen_serial".into()})
        };

        let mut reader = BufReader::with_capacity(1, opened_port);
        let thread_handle = thread::spawn(move || {
            loop {
                let mut buf = String::new();
                let result = reader.read_line(&mut buf);
                match result {
                    Ok(_) => {
                        handle.emit_all("ratpad://serial", SerialEvent::Event(buf)).unwrap();
                    },
                    Err(error) => match error.kind() {
                        ErrorKind::BrokenPipe => break,
                        _ => ()
                    }
                }
            }
            handle.emit_all("ratpad://serial", SerialEvent::Disconnect).unwrap();
        });
        Ok(thread_handle)
    }
}