import time
from adafruit_macropad import MacroPad, Keycode
import board
import busio
import json
from .keymap import Key, Keys
import usb_cdc
from .mode import Mode, ModeManager
from .display import DisplayManager

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
        self.mode: Mode | None = None
        self.pad = MacroPad()
        self.encoder_switch = self.pad.encoder_switch
        self.encoder_rotation = self.pad.encoder
        self.serial = usb_cdc.data
        self.serial.timeout = 0
        self.modes = ModeManager()
        self.display = DisplayManager(self.pad, self.modes)

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
            packet = {"mode": self.mode.key, "type": "key", "key": key.as_dict()}
        elif encoder_switch != None:
            packet = {
                "mode": self.mode.key,
                "type": "encoder.switch",
                "pressed": encoder_switch,
            }
        elif encoder_value != None:
            packet = {
                "mode": self.mode.key,
                "type": "encoder.value",
                "value": encoder_value,
            }
        else:
            return

        self.send_packet("event", packet)

    def run(self):
        self.send_packet("connect")
        self.display.refresh()
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
                        if key.special:
                            if self.mode:
                                if key == Keys.PREV:
                                    self.mode = self.modes.previous(self.mode)
                                elif key == Keys.NEXT:
                                    self.mode = self.modes.next(self.mode)
                                else:
                                    self.mode = None

                                self.display.set_mode(
                                    self.mode.key if self.mode else None
                                )
                            else:
                                if key == Keys.PREV:
                                    self.display.prev_page()
                                elif key == Keys.NEXT:
                                    self.display.next_page()
                                else:
                                    self.mode = self.modes.get(self.display.last_mode)
                                    self.display.set_mode(
                                        self.mode.key if self.mode else None
                                    )

                        else:
                            if self.mode:
                                key_info = self.mode[key.code - 3]
                                if key_info:
                                    if key_info["keys"]:
                                        self.pad.keyboard.send(
                                            *[
                                                getattr(Keycode, i.upper())
                                                for i in key_info["keys"].split("+")
                                            ]
                                        )
                                    else:
                                        self.send_event(key=key)
                            else:
                                resolved = self.display.resolve_mode(key)
                                if resolved:
                                    self.mode = resolved
                                    self.display.set_mode(self.mode.key)

                if self.mode:
                    if self.pad.encoder_switch != self.encoder_switch:
                        self.encoder_switch = self.pad.encoder_switch
                        self.send_event(encoder_switch=self.encoder_switch)

                    if self.pad.encoder != self.encoder_rotation:
                        self.encoder_rotation = self.pad.encoder
                        self.send_event(encoder_value=self.encoder_rotation)

                time.sleep(0.1)
        finally:
            self.send_packet("disconnect")
