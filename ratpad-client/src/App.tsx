import { Button, MantineProvider } from "@mantine/core";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

export default function App() {
  useEffect(() => {
    listen("ratpad://serial", console.log);
  }, []);

  return (
    <MantineProvider>
      <Button onClick={() => invoke("list_ports").then(console.log)}>
        TEST
      </Button>
    </MantineProvider>
  );
}
