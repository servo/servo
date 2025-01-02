# Licensed to the Software Freedom Conservancy (SFC) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The SFC licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

"""The Keys implementation."""

from collections import OrderedDict
from inspect import getmembers


class Keys(object):
    """
    Set of special keys codes.

    See also https://w3c.github.io/webdriver/#keyboard-actions
    """

    NULL = u"\ue000"
    CANCEL = u"\ue001"  # ^break
    HELP = u"\ue002"
    BACKSPACE = u"\ue003"
    TAB = u"\ue004"
    CLEAR = u"\ue005"
    RETURN = u"\ue006"
    ENTER = u"\ue007"
    SHIFT = u"\ue008"
    CONTROL = u"\ue009"
    ALT = u"\ue00a"
    PAUSE = u"\ue00b"
    ESCAPE = u"\ue00c"
    SPACE = u"\ue00d"
    PAGE_UP = u"\ue00e"
    PAGE_DOWN = u"\ue00f"
    END = u"\ue010"
    HOME = u"\ue011"
    LEFT = u"\ue012"
    UP = u"\ue013"
    RIGHT = u"\ue014"
    DOWN = u"\ue015"
    INSERT = u"\ue016"
    DELETE = u"\ue017"
    SEMICOLON = u"\ue018"
    EQUALS = u"\ue019"

    NUMPAD0 = u"\ue01a"  # number pad keys
    NUMPAD1 = u"\ue01b"
    NUMPAD2 = u"\ue01c"
    NUMPAD3 = u"\ue01d"
    NUMPAD4 = u"\ue01e"
    NUMPAD5 = u"\ue01f"
    NUMPAD6 = u"\ue020"
    NUMPAD7 = u"\ue021"
    NUMPAD8 = u"\ue022"
    NUMPAD9 = u"\ue023"
    MULTIPLY = u"\ue024"
    ADD = u"\ue025"
    SEPARATOR = u"\ue026"
    SUBTRACT = u"\ue027"
    DECIMAL = u"\ue028"
    DIVIDE = u"\ue029"

    F1 = u"\ue031"  # function  keys
    F2 = u"\ue032"
    F3 = u"\ue033"
    F4 = u"\ue034"
    F5 = u"\ue035"
    F6 = u"\ue036"
    F7 = u"\ue037"
    F8 = u"\ue038"
    F9 = u"\ue039"
    F10 = u"\ue03a"
    F11 = u"\ue03b"
    F12 = u"\ue03c"

    META = u"\ue03d"

    # More keys from webdriver spec
    ZENKAKUHANKAKU = u"\uE040"
    R_SHIFT = u"\uE050"
    R_CONTROL = u"\uE051"
    R_ALT = u"\uE052"
    R_META = u"\uE053"
    R_PAGEUP = u"\uE054"
    R_PAGEDOWN = u"\uE055"
    R_END = u"\uE056"
    R_HOME = u"\uE057"
    R_ARROWLEFT = u"\uE058"
    R_ARROWUP = u"\uE059"
    R_ARROWRIGHT = u"\uE05A"
    R_ARROWDOWN = u"\uE05B"
    R_INSERT = u"\uE05C"
    R_DELETE = u"\uE05D"


ALL_KEYS = getmembers(Keys, lambda x: type(x) is str)

ALL_EVENTS = OrderedDict(
    [
        ("ADD", OrderedDict(
            [
                ("code", "NumpadAdd"),
                ("ctrl", False),
                ("key", "+"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue025")
            ]
        )),
        ("ALT", OrderedDict(
            [
                ("code", "AltLeft"),
                ("ctrl", False),
                ("key", "Alt"),
                ("location", 1),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00a")
            ]
        )),
        ("BACKSPACE", OrderedDict(
            [
                ("code", "Backspace"),
                ("ctrl", False),
                ("key", "Backspace"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue003")
            ]
        )),
        ("CANCEL", OrderedDict(
            [
                ("code", ""),
                ("ctrl", False),
                ("key", "Cancel"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue001")
            ]
        )),
        ("CLEAR", OrderedDict(
            [
                ("code", ""),
                ("ctrl", False),
                ("key", "Clear"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue005")
            ]
        )),
        ("CONTROL", OrderedDict(
            [
                ("code", "ControlLeft"),
                ("ctrl", True),
                ("key", "Control"),
                ("location", 1),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue009")
            ]
        )),
        ("DECIMAL", OrderedDict(
            [
                ("code", "NumpadDecimal"),
                ("ctrl", False),
                ("key", "."),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue028")
            ]
        )),
        ("DELETE", OrderedDict(
            [
                ("code", "Delete"),
                ("ctrl", False),
                ("key", "Delete"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue017")
            ]
        )),
        ("DIVIDE", OrderedDict(
            [
                ("code", "NumpadDivide"),
                ("ctrl", False),
                ("key", "/"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue029")
            ]
        )),
        ("DOWN", OrderedDict(
            [
                ("code", "ArrowDown"),
                ("ctrl", False),
                ("key", "ArrowDown"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue015")
            ]
        )),
        ("END", OrderedDict(
            [
                ("code", "End"),
                ("ctrl", False),
                ("key", "End"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue010")
            ]
        )),
        ("ENTER", OrderedDict(
            [
                ("code", "NumpadEnter"),
                ("ctrl", False),
                ("key", "Enter"),
                ("location", 1),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue007")
            ]
        )),
        ("EQUALS", OrderedDict(
            [
                ("code", "NumpadEqual"),
                ("ctrl", False),
                ("key", "="),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue019")
            ]
        )),
        ("ESCAPE", OrderedDict(
            [
                ("code", "Escape"),
                ("ctrl", False),
                ("key", "Escape"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00c")
            ]
        )),
        ("F1", OrderedDict(
            [
                ("code", "F1"),
                ("ctrl", False),
                ("key", "F1"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue031")
            ]
        )),
        ("F10", OrderedDict(
            [
                ("code", "F10"),
                ("ctrl", False),
                ("key", "F10"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue03a")
            ]
        )),
        ("F11", OrderedDict(
            [
                ("code", "F11"),
                ("ctrl", False),
                ("key", "F11"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue03b")
            ]
        )),
        ("F12", OrderedDict(
            [
                ("code", "F12"),
                ("ctrl", False),
                ("key", "F12"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue03c")
            ]
        )),
        ("F2", OrderedDict(
            [
                ("code", "F2"),
                ("ctrl", False),
                ("key", "F2"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue032")
            ]
        )),
        ("F3", OrderedDict(
            [
                ("code", "F3"),
                ("ctrl", False),
                ("key", "F3"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue033")
            ]
        )),
        ("F4", OrderedDict(
            [
                ("code", "F4"),
                ("ctrl", False),
                ("key", "F4"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue034")
            ]
        )),
        ("F5", OrderedDict(
            [
                ("code", "F5"),
                ("ctrl", False),
                ("key", "F5"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue035")
            ]
        )),
        ("F6", OrderedDict(
            [
                ("code", "F6"),
                ("ctrl", False),
                ("key", "F6"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue036")
            ]
        )),
        ("F7", OrderedDict(
            [
                ("code", "F7"),
                ("ctrl", False),
                ("key", "F7"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue037")
            ]
        )),
        ("F8", OrderedDict(
            [
                ("code", "F8"),
                ("ctrl", False),
                ("key", "F8"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue038")
            ]
        )),
        ("F9", OrderedDict(
            [
                ("code", "F9"),
                ("ctrl", False),
                ("key", "F9"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue039")
            ]
        )),
        ("HELP", OrderedDict(
            [
                ("code", "Help"),
                ("ctrl", False),
                ("key", "Help"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue002")
            ]
        )),
        ("HOME", OrderedDict(
            [
                ("code", "Home"),
                ("ctrl", False),
                ("key", "Home"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue011")
            ]
        )),
        ("INSERT", OrderedDict(
            [
                ("code", "Insert"),
                ("ctrl", False),
                ("key", "Insert"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue016")
            ]
        )),
        ("LEFT", OrderedDict(
            [
                ("code", "ArrowLeft"),
                ("ctrl", False),
                ("key", "ArrowLeft"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue012")
            ]
        )),
        ("META", OrderedDict(
            [
                ("code", "MetaLeft"),
                ("ctrl", False),
                ("key", "Meta"),
                ("location", 1),
                ("meta", True),
                ("shift", False),
                ("value", u"\ue03d")
            ]
        )),
        ("MULTIPLY", OrderedDict(
            [
                ("code", "NumpadMultiply"),
                ("ctrl", False),
                ("key", "*"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue024")
            ]
        )),
        ("NULL", OrderedDict(
            [
                ("code", ""),
                ("ctrl", False),
                ("key", "Unidentified"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue000")
            ]
        )),
        ("NUMPAD0", OrderedDict(
            [
                ("code", "Numpad0"),
                ("ctrl", False),
                ("key", "0"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01a")
            ]
        )),
        ("NUMPAD1", OrderedDict(
            [
                ("code", "Numpad1"),
                ("ctrl", False),
                ("key", "1"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01b")
            ]
        )),
        ("NUMPAD2", OrderedDict(
            [
                ("code", "Numpad2"),
                ("ctrl", False),
                ("key", "2"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01c")
            ]
        )),
        ("NUMPAD3", OrderedDict(
            [
                ("code", "Numpad3"),
                ("ctrl", False),
                ("key", "3"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01d")
            ]
        )),
        ("NUMPAD4", OrderedDict(
            [
                ("code", "Numpad4"),
                ("ctrl", False),
                ("key", "4"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01e")
            ]
        )),
        ("NUMPAD5", OrderedDict(
            [
                ("code", "Numpad5"),
                ("ctrl", False),
                ("key", "5"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue01f")
            ]
        )),
        ("NUMPAD6", OrderedDict(
            [
                ("code", "Numpad6"),
                ("ctrl", False),
                ("key", "6"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue020")
            ]
        )),
        ("NUMPAD7", OrderedDict(
            [
                ("code", "Numpad7"),
                ("ctrl", False),
                ("key", "7"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue021")
            ]
        )),
        ("NUMPAD8", OrderedDict(
            [
                ("code", "Numpad8"),
                ("ctrl", False),
                ("key", "8"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue022")
            ]
        )),
        ("NUMPAD9", OrderedDict(
            [
                ("code", "Numpad9"),
                ("ctrl", False),
                ("key", "9"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue023")
            ]
        )),
        ("PAGE_DOWN", OrderedDict(
            [
                ("code", "PageDown"),
                ("ctrl", False),
                ("key", "PageDown"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00f")
            ]
        )),
        ("PAGE_UP", OrderedDict(
            [
                ("code", "PageUp"),
                ("ctrl", False),
                ("key", "PageUp"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00e")
            ]
        )),
        ("PAUSE", OrderedDict(
            [
                ("code", "Pause"),
                ("ctrl", False),
                ("key", "Pause"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00b")
            ]
        )),
        ("RETURN", OrderedDict(
            [
                ("code", "Enter"),
                ("ctrl", False),
                ("key", "Enter"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue006")
            ]
        )),
        ("RIGHT", OrderedDict(
            [
                ("code", "ArrowRight"),
                ("ctrl", False),
                ("key", "ArrowRight"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue014")
            ]
        )),
        ("R_ALT", OrderedDict(
            [
                ("code", "AltRight"),
                ("ctrl", False),
                ("key", "Alt"),
                ("location", 2),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue052")
            ]
        )),
        ("R_ARROWDOWN", OrderedDict(
            [
                ("code", "Numpad2"),
                ("ctrl", False),
                ("key", "ArrowDown"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue05b")
            ]
        )),
        ("R_ARROWLEFT", OrderedDict(
            [
                ("code", "Numpad4"),
                ("ctrl", False),
                ("key", "ArrowLeft"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue058")
            ]
        )),
        ("R_ARROWRIGHT", OrderedDict(
            [
                ("code", "Numpad6"),
                ("ctrl", False),
                ("key", "ArrowRight"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue05a")
            ]
        )),
        ("R_ARROWUP", OrderedDict(
            [
                ("code", "Numpad8"),
                ("ctrl", False),
                ("key", "ArrowUp"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue059")
            ]
        )),
        ("R_CONTROL", OrderedDict(
            [
                ("code", "ControlRight"),
                ("ctrl", True),
                ("key", "Control"),
                ("location", 2),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue051")
            ]
        )),
        ("R_DELETE", OrderedDict(
            [
                ("code", "NumpadDecimal"),
                ("ctrl", False),
                ("key", "Delete"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue05d")
            ]
        )),
        ("R_END", OrderedDict(
            [
                ("code", "Numpad1"),
                ("ctrl", False),
                ("key", "End"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue056")
            ]
        )),
        ("R_HOME", OrderedDict(
            [
                ("code", "Numpad7"),
                ("ctrl", False),
                ("key", "Home"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue057")
            ]
        )),
        ("R_INSERT", OrderedDict(
            [
                ("code", "Numpad0"),
                ("ctrl", False),
                ("key", "Insert"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue05c")
            ]
        )),
        ("R_META", OrderedDict(
            [
                ("code", "MetaRight"),
                ("ctrl", False),
                ("key", "Meta"),
                ("location", 2),
                ("meta", True),
                ("shift", False),
                ("value", u"\ue053")
            ]
        )),
        ("R_PAGEDOWN", OrderedDict(
            [
                ("code", "Numpad3"),
                ("ctrl", False),
                ("key", "PageDown"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue055")
            ]
        )),
        ("R_PAGEUP", OrderedDict(
            [
                ("code", "Numpad9"),
                ("ctrl", False),
                ("key", "PageUp"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue054")
            ]
        )),
        ("R_SHIFT", OrderedDict(
            [
                ("code", "ShiftRight"),
                ("ctrl", False),
                ("key", "Shift"),
                ("location", 2),
                ("meta", False),
                ("shift", True),
                ("value", u"\ue050")
            ]
        )),
        ("SEMICOLON", OrderedDict(
            [
                ("code", ""),
                ("ctrl", False),
                ("key", ";"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue018")
            ]
        )),
        ("SEPARATOR", OrderedDict(
            [
                ("code", "NumpadComma"),
                ("ctrl", False),
                ("key", ","),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue026")
            ]
        )),
        ("SHIFT", OrderedDict(
            [
                ("code", "ShiftLeft"),
                ("ctrl", False),
                ("key", "Shift"),
                ("location", 1),
                ("meta", False),
                ("shift", True),
                ("value", u"\ue008")
            ]
        )),
        ("SPACE", OrderedDict(
            [
                ("code", "Space"),
                ("ctrl", False),
                ("key", " "),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue00d")
            ]
        )),
        ("SUBTRACT", OrderedDict(
            [
                ("code", "NumpadSubtract"),
                ("ctrl", False),
                ("key", "-"),
                ("location", 3),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue027")
            ]
        )),
        ("TAB", OrderedDict(
            [
                ("code", "Tab"),
                ("ctrl", False),
                ("key", "Tab"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue004")
            ]
        )),
        ("UP", OrderedDict(
            [
                ("code", "ArrowUp"),
                ("ctrl", False),
                ("key", "ArrowUp"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue013")
            ]
        )),
        ("ZENKAKUHANKAKU", OrderedDict(
            [
                ("code", ""),
                ("ctrl", False),
                ("key", "ZenkakuHankaku"),
                ("location", 0),
                ("meta", False),
                ("shift", False),
                ("value", u"\ue040")
            ]
        ))
    ]
)

ALTERNATIVE_KEY_NAMES = {
    "ADD": "Add",
    "DECIMAL": "Decimal",
    "DELETE": "Del",
    "DIVIDE": "Divide",
    "DOWN": "Down",
    "ESCAPE": "Esc",
    "LEFT": "Left",
    "MULTIPLY": "Multiply",
    "R_ARROWDOWN": "Down",
    "R_ARROWLEFT": "Left",
    "R_ARROWRIGHT": "Right",
    "R_ARROWUP": "Up",
    "R_DELETE": "Del",
    "RIGHT": "Right",
    "SEPARATOR": "Separator",
    "SPACE": "Spacebar",
    "SUBTRACT": "Subtract",
    "UP": "Up",
}
