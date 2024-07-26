import time
from adafruit_macropad import MacroPad
import board
import busio
import json
from .keymap import Key, Keys
import usb_cdc

try:
    from typing import Literal, Any
except ImportError:
    pass


class CommandPacket:
    def __init__(self, data: bytes) -> None:
        decoded = data.decode()
        command = decoded.split(":")[0]
        rest = ":".join(decoded.split(":")[1:])
        self.command = command.lower()
        if len(rest) > 0:
            try:
                self.data = json.loads(rest.strip().strip(";"))
            except:
                self.data = None
        else:
            self.data = None


class PadManager:
    def __init__(self):
        self.mode = "base"
        self.pad = MacroPad()
        self.encoder_switch = self.pad.encoder_switch
        self.encoder_rotation = self.pad.encoder
        self.serial = usb_cdc.data
        self.serial.timeout = 0

    def send_packet(self, type: str, data: Any | None = None):
        self.serial.write(
            type.upper().encode()
            + b":"
            + (json.dumps(data).encode() if data else b"")
            + b";\n"
        )

    def parse_packet(self, data: bytes) -> CommandPacket:
        return CommandPacket(data)

    def send_event(
        self,
        key: Key | None = None,
        encoder_switch: bool | None = None,
        encoder_value: int | None = None,
    ):
        if key:
            packet = {"mode": self.mode, "type": "key", "key": key.as_dict()}
        elif encoder_switch != None:
            packet = {
                "mode": self.mode,
                "type": "encoder.switch",
                "pressed": encoder_switch,
            }
        elif encoder_value != None:
            packet = {
                "mode": self.mode,
                "type": "encoder.value",
                "value": encoder_value,
            }
        else:
            return

        self.send_packet("event", packet)

    def run(self):
        self.send_packet("connect")
        try:
            while True:
                line = self.serial.readline()
                if line:
                    if line.strip().endswith(b";"):
                        command = self.parse_packet(line)
                        self.send_packet(
                            "recv", {"command": command.command, "data": command.data}
                        )
                if bool(self.pad.keys.events):
                    event = self.pad.keys.events.get()
                    key = Keys.get(event.key_number)
                    if event.pressed:
                        if not key.special:
                            self.send_event(key=key)

                if self.pad.encoder_switch != self.encoder_switch:
                    self.encoder_switch = self.pad.encoder_switch
                    self.send_event(encoder_switch=self.encoder_switch)

                if self.pad.encoder != self.encoder_rotation:
                    self.encoder_rotation = self.pad.encoder
                    self.send_event(encoder_value=self.encoder_rotation)

                time.sleep(0.1)
        finally:
            self.send_packet("disconnect")
