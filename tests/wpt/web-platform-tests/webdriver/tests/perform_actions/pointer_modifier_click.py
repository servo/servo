import pytest

from tests.perform_actions.support.keys import Keys
from tests.perform_actions.support.refine import get_events
from tests.support.helpers import filter_dict


@pytest.mark.parametrize("modifier, prop", [
   (Keys.ALT, "altKey"),
   (Keys.R_ALT, "altKey"),
   (Keys.META, "metaKey"),
   (Keys.R_META, "metaKey"),
   (Keys.SHIFT, "shiftKey"),
   (Keys.R_SHIFT, "shiftKey"),
])
def test_modifier_click(session, test_actions_page, key_chain, mouse_chain, modifier, prop):
    key_chain \
        .pause(200) \
        .key_down(modifier) \
        .pause(200) \
        .key_up(modifier)
    outer = session.find.css("#outer", all=False)
    mouse_chain.click(element=outer)
    session.actions.perform([key_chain.dict, mouse_chain.dict])
    expected = [
        {"type": "mousemove"},
        {"type": "mousedown"},
        {"type": "mouseup"},
        {"type": "click"},
    ]
    defaults = {
        "altKey": False,
        "metaKey": False,
        "shiftKey": False,
        "ctrlKey": False
    }
    for e in expected:
        e.update(defaults)
        if e["type"] != "mousemove":
            e[prop] = True
    filtered_events = [filter_dict(e, expected[0]) for e in get_events(session)]
    assert expected == filtered_events


def test_many_modifiers_click(session, test_actions_page, key_chain, mouse_chain):
    outer = session.find.css("#outer", all=False)
    dblclick_timeout = 800
    key_chain \
        .pause(0) \
        .key_down(Keys.ALT) \
        .key_down(Keys.SHIFT) \
        .pause(dblclick_timeout) \
        .key_up(Keys.ALT) \
        .key_up(Keys.SHIFT)
    mouse_chain \
        .pointer_move(0, 0, origin=outer) \
        .pause(0) \
        .pointer_down() \
        .pointer_up() \
        .pause(0) \
        .pause(0) \
        .pointer_down()
    session.actions.perform([key_chain.dict, mouse_chain.dict])
    expected = [
        {"type": "mousemove"},
        # shift and alt pressed
        {"type": "mousedown"},
        {"type": "mouseup"},
        {"type": "click"},
        # no modifiers pressed
        {"type": "mousedown"},
    ]
    defaults = {
        "altKey": False,
        "metaKey": False,
        "shiftKey": False,
        "ctrlKey": False
    }
    for e in expected:
        e.update(defaults)
    for e in expected[1:4]:
        e["shiftKey"] = True
        e["altKey"] = True
    events = [filter_dict(e, expected[0]) for e in get_events(session)]
    assert events == expected
