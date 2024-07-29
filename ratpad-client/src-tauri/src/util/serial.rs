pub mod serial_client {
    use serde::Serialize;
    use tokio_serial::{available_ports, SerialPortType, UsbPortInfo};

    #[derive(Serialize)]
    pub enum SerialErrorType {
        List
    }

    #[derive(Serialize)]
    pub struct SerialError {
        error_type: SerialErrorType,
        error_source: String,
    }

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

    pub fn get_ports() -> Result<Vec<PortInfo>, SerialError> {
        let result = available_ports();
        match result {
            Ok(ports) => Ok(ports.iter().map(|r| PortInfo {port_name: r.port_name.clone(), port_type: r.port_type.clone()} ).collect()),
            Err(_) => Err(SerialError { error_type: SerialErrorType::List, error_source: "get_ports".to_string() })
        }
    }
}