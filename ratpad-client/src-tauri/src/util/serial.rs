pub mod serial_client {
    use std::{
        error::Error,
        fmt,
        io::{BufRead, BufReader, BufWriter, ErrorKind, Write},
        sync::mpsc,
        thread::{self, JoinHandle},
        time::Duration,
    };

    use serde::{Deserialize, Serialize};
    use tauri::{App, Manager};
    use tokio_serial::{available_ports, new, SerialPort, SerialPortType, UsbPortInfo};

    use crate::ratpad_communication::{create_message, parse_message, Message};

    #[derive(Serialize, Debug)]
    pub enum SerialErrorType {
        List,
        Connection(String),
    }

    #[derive(Serialize, Debug)]
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
        Unknown,
    }

    #[derive(Serialize)]
    pub struct PortInfo {
        port_name: String,

        #[serde(with = "SerialPortTypeDef")]
        port_type: SerialPortType,
    }

    #[derive(Serialize, Clone)]
    pub enum SerialEvent {
        Event(Option<Message>),
        Connect,
        Disconnect,
    }

    pub fn get_ports() -> Result<Vec<PortInfo>, SerialError> {
        let result = available_ports();
        match result {
            Ok(ports) => Ok(ports
                .iter()
                .map(|r| PortInfo {
                    port_name: r.port_name.clone(),
                    port_type: r.port_type.clone(),
                })
                .collect()),
            Err(_) => Err(SerialError {
                error_type: SerialErrorType::List,
                error_source: "get_ports".to_string(),
            }),
        }
    }

    #[derive(Serialize, Clone, Deserialize, Debug)]
    pub enum ListenerCommand {
        Disconnect,
        Connect { new_port: String, new_rate: u32 },
        Quit,
        Send(Message),
    }

    struct ListenerState {
        port: String,
        rate: u32,
        reader: Option<BufReader<Box<dyn SerialPort>>>,
        writer: Option<BufWriter<Box<dyn SerialPort>>>
    }

    pub fn start_serial_listener(app: &mut App) -> JoinHandle<()> {
        let (tx, rx) = mpsc::channel::<ListenerCommand>();

        app.listen_global("ratpad://serial/cmd", move |event| {
            if let Some(command) = event.payload() {
                if let Ok(parsed) = serde_json::from_str::<ListenerCommand>(command) {
                    let _ = tx.send(parsed);
                } else {
                    println!("PARSE FAIL");
                }
            }
        });

        let handle = app.handle();

        let thread_result = thread::spawn(move || {
            let mut state: Option<ListenerState> = None;

            loop {
                if let Ok(command) = rx.try_recv() {
                    match command {
                        ListenerCommand::Disconnect => {
                            state = None;
                            handle
                                .emit_all("ratpad://serial", SerialEvent::Disconnect)
                                .expect("CRITICAL: emit-failure");
                        }
                        ListenerCommand::Connect { new_port, new_rate } => {
                            let builder =
                                new(new_port.clone(), new_rate).timeout(Duration::from_millis(100));
                            let opened = builder.open();
                            if let Ok(serial) = opened {
                                state = Some(ListenerState {
                                    port: new_port,
                                    rate: new_rate,
                                    reader: Some(BufReader::with_capacity(1, serial.try_clone().unwrap())),
                                    writer: Some(BufWriter::with_capacity(1, serial.try_clone().unwrap()))
                                });
                                handle
                                    .emit_all("ratpad://serial", SerialEvent::Connect)
                                    .expect("CRITICAL: emit-failure");
                            } else if let Err(_) = opened {
                                state = Some(ListenerState {
                                    port: new_port,
                                    rate: new_rate,
                                    reader: None,
                                    writer: None
                                });
                            }
                        }
                        ListenerCommand::Quit => break,
                        ListenerCommand::Send(msg) => {
                            if let Some(ref mut st) = state {
                                if let Some(ref mut writer) = st.writer {
                                    if let Ok(ser) = create_message(msg.header, msg.data) {
                                        match writer.write_all(ser.as_bytes()) {
                                            Ok(_) => {
                                                let _ = writer.flush();
                                            },
                                            Err(_) => ()
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                match state {
                    Some(ref mut st) => {
                        if st.reader.is_some() {
                            let mut failure = false;
                            for line in st.reader.as_mut().unwrap().lines() {
                                match line {
                                    Ok(ref read) => {
                                        handle
                                            .emit_all(
                                                "ratpad://serial",
                                                SerialEvent::Event(parse_message(read.clone())),
                                            )
                                            .expect("CRITICAL: emit-failure");
                                        
                                    }
                                    Err(ref error) => match error.kind() {
                                        ErrorKind::BrokenPipe => {
                                            failure = true;
                                            break;
                                        },
                                        ErrorKind::TimedOut => {
                                            break;
                                        },
                                        _ => ()
                                    },
                                }
                            }

                            if failure {
                                st.writer = None;
                                st.reader = None;
                                handle
                                .emit_all("ratpad://serial", SerialEvent::Disconnect)
                                .expect("CRITICAL: emit-failure");
                            }
                        } else {
                            if let Ok(opened) = new(st.port.clone(), st.rate)
                                .timeout(Duration::from_millis(100))
                                .open()
                            {
                                st.reader = Some(BufReader::with_capacity(1, opened.try_clone().unwrap()));
                                st.writer = Some(BufWriter::with_capacity(1, opened.try_clone().unwrap()));
                                handle
                                .emit_all("ratpad://serial", SerialEvent::Connect)
                                .expect("CRITICAL: emit-failure");
                            }
                        }
                    }
                    None => (),
                }
            }
        });

        thread_result
    }
}
