import json


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


class ModeManager:
    def __init__(self):
        with open("db.json", "r") as f:
            data = json.load(f)

        self.modes = [Mode.from_entry(entry) for entry in data["modes"]]
        self.colors: dict[str, list[int]] = data["colors"]

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
