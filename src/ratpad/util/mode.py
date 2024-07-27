import json

try:
    from typing import Literal
except ImportError:
    pass


class Mode:

    def __init__(
        self,
        key: str,
        title: str,
        title_short: str,
        keys: list[str | dict[str, str] | None],
        color: list[str] | None = None,
    ):
        self.key = key
        self.title = title
        self.title_short = title_short
        self.color = color
        self.keys = [
            (
                (
                    {"label": i, "keys": None, "color": None}
                    if isinstance(i, str)
                    else {
                        "label": i["label"],
                        "keys": i.get("keys", None),
                        "color": i.get("color", None),
                    }
                )
                if i
                else None
            )
            for i in keys
        ]

    @classmethod
    def from_entry(cls, data: dict) -> "Mode":
        return cls(**data)

    def label(self, index: int) -> str:
        if len(self.keys) > index:
            return self.keys[index]["label"] if self.keys[index] else "--"
        else:
            return "--"

    def __getitem__(self, key: int) -> dict[str, str] | None:
        if len(self.keys) > key:
            return self.keys[key] if self.keys[key] else None
        else:
            return None

    def as_json(self):
        return {
            "key": self.key,
            "title": self.title,
            "title_short": self.title_short,
            "color": self.color,
            "keys": self.keys,
        }


class ModeManager:
    def __init__(self):
        try:
            with open("db.json", "r") as f:
                data = json.load(f)
        except:
            with open("db.json", "w") as f:
                json.dump(
                    {
                        "colors": {
                            "next": [0, 0, 0],
                            "previous": [0, 0, 0],
                            "select": [0, 0, 0],
                            "brightness": 1,
                        },
                        "modes": [],
                    },
                    f,
                )

            with open("db.json", "r") as f:
                data = json.load(f)

        self.modes = [Mode.from_entry(entry) for entry in data["modes"]]
        self.colors: dict[str, list[int]] = data["colors"]

    def as_dict(self):
        return {"colors": self.colors, "modes": [i.as_json() for i in self.modes]}

    def save(self):
        with open("db.json", "w") as f:
            json.dump(self.as_dict(), f)

    @property
    def mode_mapping(self) -> dict[str, Mode]:
        return {entry.key: entry for entry in self.modes}

    def get(self, key: str) -> Mode | None:
        return self.mode_mapping.get(key, None)

    def mode_index(self, mode: Mode) -> int:
        res = [i for i in range(len(self.modes)) if self.modes[i].key == mode.key]
        if len(res) > 0:
            return res[0]
        else:
            return None

    def next(self, mode: Mode) -> Mode:
        current = self.mode_index(mode)
        if current == None:
            return mode
        next_index = current + 1
        if next_index >= len(self.modes):
            return self.modes[0]
        return self.modes[next_index]

    def previous(self, mode: Mode) -> Mode:
        current = self.mode_index(mode)
        if current == None:
            return mode
        next_index = current - 1
        if next_index < 0:
            return self.modes[len(self.modes) - 1]
        return self.modes[next_index]

    def write_mode(self, mode: Mode):
        if mode.key in [i.key for i in self.modes]:
            self.modes[[i.key for i in self.modes].index(mode.key)] = mode
        else:
            self.modes.append(mode)

        self.save()

    def delete_mode(self, key: str):
        self.modes = [i for i in self.modes if i.key != key]
        self.save()

    def set_color(
        self,
        key: Literal["previous", "next", "select", "brightness"],
        value: list[int] | float,
    ):
        self.colors[key] = value
        self.save()

    def clear(self):
        self.modes = []
        self.save()
