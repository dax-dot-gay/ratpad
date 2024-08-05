import { Button, MantineProvider } from "@mantine/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { executeCommand, ConfWriteMode } from "./api/types/commands";

export default function App() {
    useEffect(() => {
        listen("ratpad://serial", console.log);
    }, []);

    return (
        <MantineProvider>
            <Button
                onClick={() =>
                    executeCommand<ConfWriteMode>({
                        type: "config.write_mode",
                        mode: {
                            key: "workspaces",
                            title: "Workspaces",
                            title_short: "WKSP",
                            color: [255, 0, 0],
                            keys: [
                                {
                                    label: "01",
                                    action: {
                                        type: "keypress",
                                        key: "a",
                                    },
                                    color: [0, 255, 0],
                                },
                            ],
                        },
                    }).then(console.log)
                }
            >
                TEST
            </Button>
        </MantineProvider>
    );
}
