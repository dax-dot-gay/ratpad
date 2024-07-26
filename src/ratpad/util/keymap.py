from microcontroller import Pin
import board


class Key:
    def __init__(self, code: int, name: str, pin: Pin, special: bool = False):
        self.code = code
        self.name = name
        self.pin = pin
        self.special = special

    def __str__(self) -> str:
        return self.name

    def __int__(self) -> int:
        return self.code

    def __eq__(self, value: object) -> bool:
        return self.code == value.code and self.name == value.name

    def as_dict(self):
        return {"code": self.code, "name": self.name}


class Keys:
    PREV = Key(0, "previous", board.KEY1, special=True)
    SELECT = Key(1, "select", board.KEY2, special=True)
    NEXT = Key(2, "next", board.KEY3, special=True)

    ACTION_1 = Key(3, "action_1", board.KEY4)
    ACTION_2 = Key(4, "action_2", board.KEY5)
    ACTION_3 = Key(5, "action_3", board.KEY6)
    ACTION_4 = Key(6, "action_4", board.KEY7)
    ACTION_5 = Key(7, "action_5", board.KEY8)
    ACTION_6 = Key(8, "action_6", board.KEY9)
    ACTION_7 = Key(9, "action_7", board.KEY10)
    ACTION_8 = Key(10, "action_8", board.KEY11)
    ACTION_9 = Key(11, "action_9", board.KEY12)

    @classmethod
    def get(self, key: str | int | Pin) -> Key | None:
        if isinstance(key, str):
            attr = "name"
        elif isinstance(key, int):
            attr = "code"
        elif isinstance(key, Pin):
            attr = "pin"

        KEYNAMES = ["PREV", "SELECT", "NEXT"]
        KEYNAMES.extend(["ACTION_" + str(i) for i in range(1, 10)])

        for a in KEYNAMES:
            if getattr(getattr(self, a), attr) == key:
                return getattr(self, a)
        return None
