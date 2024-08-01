import { Button, MantineProvider } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { executeCommand, SerialPortList } from "./api/types/commands";

export default function App() {
    useEffect(() => {
        listen("ratpad://serial", console.log);
    }, []);

    return (
        <MantineProvider>
            <Button
                onClick={() =>
                    executeCommand<SerialPortList>({
                        type: "serial.list_ports",
                        data: null,
                    }).then(console.log)
                }
            >
                TEST
            </Button>
        </MantineProvider>
    );
}
