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

"""
The Keys implementation.
"""

from inspect import getmembers


class Keys(object):
    """
    Set of special keys codes.

    See also https://w3c.github.io/webdriver/webdriver-spec.html#h-keyboard-actions
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


ALL_KEYS = getmembers(Keys, lambda x: type(x) == unicode)

ALL_EVENTS = {
    "ADD": {
        "code": "",
        "ctrl": False,
        "key": "+",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue025",
        "which": 0,
    },
    "ALT": {
        "code": "AltLeft",
        "ctrl": False,
        "key": "Alt",
        "location": 1,
        "meta": False,
        "shift": False,
        "value": u"\ue00a",
        "which": 0,
    },
    "BACKSPACE": {
        "code": "Backspace",
        "ctrl": False,
        "key": "Backspace",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue003",
        "which": 0,
    },
    "CANCEL": {
        "code": "",
        "ctrl": False,
        "key": "Cancel",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue001",
        "which": 0,
    },
    "CLEAR": {
        "code": "",
        "ctrl": False,
        "key": "Clear",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue005",
        "which": 0,
    },
    "CONTROL": {
        "code": "ControlLeft",
        "ctrl": True,
        "key": "Control",
        "location": 1,
        "meta": False,
        "shift": False,
        "value": u"\ue009",
        "which": 0,
    },
    "DECIMAL": {
        "code": "NumpadDecimal",
        "ctrl": False,
        "key": ".",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue028",
        "which": 0,
    },
    "DELETE": {
        "code": "Delete",
        "ctrl": False,
        "key": "Delete",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue017",
        "which": 0,
    },
    "DIVIDE": {
        "code": "NumpadDivide",
        "ctrl": False,
        "key": "/",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue029",
        "which": 0,
    },
    "DOWN": {
        "code": "ArrowDown",
        "ctrl": False,
        "key": "ArrowDown",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue015",
        "which": 0,
    },
    "END": {
        "code": "End",
        "ctrl": False,
        "key": "End",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue010",
        "which": 0,
    },
    "ENTER": {
        "code": "NumpadEnter",
        "ctrl": False,
        "key": "Enter",
        "location": 1,
        "meta": False,
        "shift": False,
        "value": u"\ue007",
        "which": 0,
    },
    "EQUALS": {
        "code": "",
        "ctrl": False,
        "key": "=",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue019",
        "which": 0,
    },
    "ESCAPE": {
        "code": "Escape",
        "ctrl": False,
        "key": "Escape",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue00c",
        "which": 0,
    },
    "F1": {
        "code": "F1",
        "ctrl": False,
        "key": "F1",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue031",
        "which": 0,
    },
    "F10": {
        "code": "F10",
        "ctrl": False,
        "key": "F10",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue03a",
        "which": 0,
    },
    "F11": {
        "code": "F11",
        "ctrl": False,
        "key": "F11",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue03b",
        "which": 0,
    },
    "F12": {
        "code": "F12",
        "ctrl": False,
        "key": "F12",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue03c",
        "which": 0,
    },
    "F2": {
        "code": "F2",
        "ctrl": False,
        "key": "F2",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue032",
        "which": 0,
    },
    "F3": {
        "code": "F3",
        "ctrl": False,
        "key": "F3",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue033",
        "which": 0,
    },
    "F4": {
        "code": "F4",
        "ctrl": False,
        "key": "F4",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue034",
        "which": 0,
    },
    "F5": {
        "code": "F5",
        "ctrl": False,
        "key": "F5",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue035",
        "which": 0,
    },
    "F6": {
        "code": "F6",
        "ctrl": False,
        "key": "F6",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue036",
        "which": 0,
    },
    "F7": {
        "code": "F7",
        "ctrl": False,
        "key": "F7",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue037",
        "which": 0,
    },
    "F8": {
        "code": "F8",
        "ctrl": False,
        "key": "F8",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue038",
        "which": 0,
    },
    "F9": {
        "code": "F9",
        "ctrl": False,
        "key": "F9",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue039",
        "which": 0,
    },
    "HELP": {
        "code": "Help",
        "ctrl": False,
        "key": "Help",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue002",
        "which": 0,
    },
    "HOME": {
        "code": "Home",
        "ctrl": False,
        "key": "Home",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue011",
        "which": 0,
    },
    "INSERT": {
        "code": "Insert",
        "ctrl": False,
        "key": "Insert",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue016",
        "which": 0,
    },
    "LEFT": {
        "code": "ArrowLeft",
        "ctrl": False,
        "key": "ArrowLeft",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue012",
        "which": 0,
    },
    "META": {
        "code": "OSLeft",
        "ctrl": False,
        "key": "Meta",
        "location": 1,
        "meta": True,
        "shift": False,
        "value": u"\ue03d",
        "which": 0,
    },
    "MULTIPLY": {
        "code": "NumpadMultiply",
        "ctrl": False,
        "key": "*",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue024",
        "which": 0,
    },
    "NULL": {
        "code": "",
        "ctrl": False,
        "key": "Unidentified",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue000",
        "which": 0,
    },
    "NUMPAD0": {
        "code": "Numpad0",
        "ctrl": False,
        "key": "0",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01a",
        "which": 0,
    },
    "NUMPAD1": {
        "code": "Numpad1",
        "ctrl": False,
        "key": "1",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01b",
        "which": 0,
    },
    "NUMPAD2": {
        "code": "Numpad2",
        "ctrl": False,
        "key": "2",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01c",
        "which": 0,
    },
    "NUMPAD3": {
        "code": "Numpad3",
        "ctrl": False,
        "key": "3",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01d",
        "which": 0,
    },
    "NUMPAD4": {
        "code": "PageDown",
        "ctrl": False,
        "key": "4",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01e",
        "which": 0,
    },
    "NUMPAD5": {
        "code": "PageUp",
        "ctrl": False,
        "key": "5",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue01f",
        "which": 0,
    },
    "NUMPAD6": {
        "code": "Numpad6",
        "ctrl": False,
        "key": "6",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue020",
        "which": 0,
    },
    "NUMPAD7": {
        "code": "Numpad7",
        "ctrl": False,
        "key": "7",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue021",
        "which": 0,
    },
    "NUMPAD8": {
        "code": "Numpad8",
        "ctrl": False,
        "key": "8",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue022",
        "which": 0,
    },
    "NUMPAD9": {
        "code": "Numpad9",
        "ctrl": False,
        "key": "9",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue023",
        "which": 0,
    },
    "PAGE_DOWN": {
        "code": "",
        "ctrl": False,
        "key": "PageDown",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue00f",
        "which": 0,
    },
    "PAGE_UP": {
        "code": "",
        "ctrl": False,
        "key": "PageUp",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue00e",
        "which": 0,
    },
    "PAUSE": {
        "code": "",
        "ctrl": False,
        "key": "Pause",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue00b",
        "which": 0,
    },
    "RETURN": {
        "code": "Enter",
        "ctrl": False,
        "key": "Return",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue006",
        "which": 0,
    },
    "RIGHT": {
        "code": "ArrowRight",
        "ctrl": False,
        "key": "ArrowRight",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue014",
        "which": 0,
    },
    "R_ALT": {
        "code": "AltRight",
        "ctrl": False,
        "key": "Alt",
        "location": 2,
        "meta": False,
        "shift": False,
        "value": u"\ue052",
        "which": 0,
    },
    "R_ARROWDOWN": {
        "code": "Numpad2",
        "ctrl": False,
        "key": "ArrowDown",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue05b",
        "which": 0,
    },
    "R_ARROWLEFT": {
        "code": "Numpad4",
        "ctrl": False,
        "key": "ArrowLeft",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue058",
        "which": 0,
    },
    "R_ARROWRIGHT": {
        "code": "Numpad6",
        "ctrl": False,
        "key": "ArrowRight",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue05a",
        "which": 0,
    },
    "R_ARROWUP": {
        "code": "Numpad8",
        "ctrl": False,
        "key": "ArrowUp",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue059",
        "which": 0,
    },
    "R_CONTROL": {
        "code": "ControlRight",
        "ctrl": True,
        "key": "Control",
        "location": 2,
        "meta": False,
        "shift": False,
        "value": u"\ue051",
        "which": 0,
    },
    "R_DELETE": {
        "code": "NumpadDecimal",
        "ctrl": False,
        "key": "Delete",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue05d",
        "which": 0,
    },
    "R_END": {
        "code": "Numpad1",
        "ctrl": False,
        "key": "End",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue056",
        "which": 0,
    },
    "R_HOME": {
        "code": "Numpad7",
        "ctrl": False,
        "key": "Home",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue057",
        "which": 0,
    },
    "R_INSERT": {
        "code": "Numpad0",
        "ctrl": False,
        "key": "Insert",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue05c",
        "which": 0,
    },
    "R_META": {
        "code": "OSRight",
        "ctrl": False,
        "key": "Meta",
        "location": 2,
        "meta": True,
        "shift": False,
        "value": u"\ue053",
        "which": 0,
    },
    "R_PAGEDOWN": {
        "code": "Numpad3",
        "ctrl": False,
        "key": "PageDown",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue055",
        "which": 0,
    },
    "R_PAGEUP": {
        "code": "Numpad9",
        "ctrl": False,
        "key": "PageUp",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue054",
        "which": 0,
    },
    "R_SHIFT": {
        "code": "ShiftRight",
        "ctrl": False,
        "key": "Shift",
        "location": 2,
        "meta": False,
        "shift": True,
        "value": u"\ue050",
        "which": 0,
    },
    "SEMICOLON": {
        "code": "",
        "ctrl": False,
        "key": ";",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue018",
        "which": 0,
    },
    "SEPARATOR": {
        "code": "NumpadSubtract",
        "ctrl": False,
        "key": ",",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue026",
        "which": 0,
    },
    "SHIFT": {
        "code": "ShiftLeft",
        "ctrl": False,
        "key": "Shift",
        "location": 1,
        "meta": False,
        "shift": True,
        "value": u"\ue008",
        "which": 0,
    },
    "SPACE": {
        "code": "Space",
        "ctrl": False,
        "key": " ",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue00d",
        "which": 0,
    },
    "SUBTRACT": {
        "code": "",
        "ctrl": False,
        "key": "-",
        "location": 3,
        "meta": False,
        "shift": False,
        "value": u"\ue027",
        "which": 0,
    },
    "TAB": {
        "code": "Tab",
        "ctrl": False,
        "key": "Tab",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue004",
        "which": 0,
    },
    "UP": {
        "code": "ArrowUp",
        "ctrl": False,
        "key": "ArrowUp",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue013",
        "which": 0,
    },
    "ZENKAKUHANKAKU": {
        "code": "",
        "ctrl": False,
        "key": "ZenkakuHankaku",
        "location": 0,
        "meta": False,
        "shift": False,
        "value": u"\ue040",
        "which": 0,
    }
}
