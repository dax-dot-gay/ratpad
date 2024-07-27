from math import ceil, floor
import math
from adafruit_macropad import MacroPad
from .mode import ModeManager, Mode
from .keymap import Key

DISPLAY_LENGTH = 20


class DisplayManager:
    def __init__(self, pad: MacroPad, mode_manager: ModeManager):
        self.pad = pad
        self.display = self.pad.display_text()
        self.current_mode = None
        self.modes = mode_manager
        self.page_number = 0
        self.last_mode = None

    @property
    def page_modes(self) -> list[Mode]:
        return self.modes.modes[self.page_number * 9 : (self.page_number + 1) * 9]

    def next_page(self):
        self.page_number += 1
        if len(self.page_modes) == 0:
            self.page_number = 0
        self.refresh()

    def prev_page(self):
        self.page_number -= 1
        if self.page_number < 0:
            self.page_number = math.floor(len(self.modes.modes) / 9)
        self.refresh()

    @property
    def mode(self) -> Mode | None:
        return self.modes.get(self.current_mode) if self.current_mode else None

    def set_mode(self, mode: str | None):
        if mode != self.current_mode:
            self.last_mode = self.current_mode
            self.current_mode = mode
            self.refresh()

    def pad_center(self, text: str, max_length: int) -> str:
        trimmed = text.strip()[:max_length]
        if len(trimmed) == max_length:
            return trimmed

        return (
            " " * ceil((max_length - len(trimmed)) / 2)
            + trimmed
            + " " * floor((max_length - len(trimmed)) / 2)
        )

    def display_home(self):
        self.display[0].text = self.pad_center(
            f"HOME: {self.page_number + 1}", DISPLAY_LENGTH
        )
        self.display[1].text = (
            "<- "
            + self.pad_center(
                "[BACK]" if self.last_mode else "[ -- ]", DISPLAY_LENGTH - 6
            )
            + " ->"
        )
        self.pad.pixels.brightness = self.modes.colors["brightness"]
        self.pad.pixels[0] = self.modes.colors["previous"]
        self.pad.pixels[1] = self.modes.colors["select"]
        self.pad.pixels[2] = self.modes.colors["next"]

        modes = self.page_modes
        buttons = []
        lc = 2
        for i in range(9):
            if i < len(modes):
                buttons.append("[" + self.pad_center(modes[i].title_short, 4) + "]")
                self.pad.pixels[3 + i] = (
                    modes[i].color if modes[i].color else self.modes.colors["default"]
                )
            else:
                buttons.append("[ -- ]")
                self.pad.pixels[3 + i] = [0, 0, 0]
            if len(buttons) == 3:
                self.display[lc].text = " ".join(buttons)
                buttons = []
                lc += 1

    def display_mode(self):
        self.display[0].text = self.pad_center(self.mode.title, DISPLAY_LENGTH)
        self.display[1].text = (
            "<- " + self.pad_center("[HOME]", DISPLAY_LENGTH - 6) + " ->"
        )

        self.pad.pixels.brightness = self.modes.colors["brightness"]
        self.pad.pixels[0] = self.modes.colors["previous"]
        self.pad.pixels[1] = self.modes.colors["select"]
        self.pad.pixels[2] = self.modes.colors["next"]

        mode = self.mode
        buttons = []
        lc = 2
        for i in range(9):
            if i < len(mode.keys):
                buttons.append("[" + self.pad_center(mode.label(i), 4) + "]")
                self.pad.pixels[3 + i] = (
                    (mode[i] if mode[i]["color"] else self.modes.colors["default"])
                    if mode[i]
                    else [0, 0, 0]
                )
            else:
                buttons.append("[ -- ]")
                self.pad.pixels[3 + i] = [0, 0, 0]
            if len(buttons) == 3:
                self.display[lc].text = " ".join(buttons)
                buttons = []
                lc += 1

    def refresh(self):
        if self.mode == None:
            self.display_home()
        else:
            self.display_mode()

        self.display.show()

    def resolve_mode(self, key: Key) -> Mode | None:
        if self.mode == None:
            modes = self.page_modes
            if key.code - 3 < len(modes):
                return modes[key.code - 3]

        return None
