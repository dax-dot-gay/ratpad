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
      <Button
        onClick={() =>
          invoke("connect", { port: "/dev/ttyACM1", rate: 115200 }).then(
            console.log
          )
        }
      >
        TEST
      </Button>
      <Button
        onClick={() =>
          invoke("send_serial", {
            message: { header: "READ_CONFIG", data: null },
          }).then(console.log)
        }
      >
        CNF
      </Button>
    </MantineProvider>
  );
}
