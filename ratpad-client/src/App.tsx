import { Button, MantineProvider } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import {
    executeCommand,
    SerialPortList,
    PadGetConfig,
} from "./api/types/commands";

export default function App() {
    useEffect(() => {
        listen("ratpad://serial", console.log);
    }, []);

    return (
        <MantineProvider>
            <Button
                onClick={() =>
                    executeCommand<PadGetConfig>({
                        type: "pad.get_config",
                        data: null,
                    }).then(console.log)
                }
            >
                TEST
            </Button>
        </MantineProvider>
    );
}
