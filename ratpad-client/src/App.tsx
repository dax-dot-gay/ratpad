import { Button, MantineProvider } from "@mantine/core";
import { invoke } from "@tauri-apps/api";

export default function App() {
  return (
    <MantineProvider>
      <Button onClick={() => invoke("list_ports").then(console.log)}>
        TEST
      </Button>
    </MantineProvider>
  );
}
