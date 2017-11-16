# META: timeout=long

import pytest

from tests.actions.support.refine import filter_dict, get_events
from tests.actions.support.keys import Keys


@pytest.mark.parametrize("modifier, prop", [
    (Keys.CONTROL, "ctrlKey"),
    (Keys.ALT, "altKey"),
    (Keys.META, "metaKey"),
    (Keys.SHIFT, "shiftKey"),
    (Keys.R_CONTROL, "ctrlKey"),
    (Keys.R_ALT, "altKey"),
    (Keys.R_META, "metaKey"),
    (Keys.R_SHIFT, "shiftKey"),
])
def test_modifier_click(session,
                       test_actions_page,
                       key_chain,
                       mouse_chain,
                       modifier,
                       prop):
    key_chain \
        .pause(0) \
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


def test_release_control_click(session, key_reporter, key_chain, mouse_chain):
    key_chain \
        .pause(0) \
        .key_down(Keys.CONTROL)
    mouse_chain \
        .pointer_move(0, 0, origin=key_reporter) \
        .pointer_down()
    session.actions.perform([key_chain.dict, mouse_chain.dict])
    session.execute_script("""
        var keyReporter = document.getElementById("keys");
        ["mousedown", "mouseup"].forEach((e) => {
            keyReporter.addEventListener(e, recordPointerEvent);
          });
        resetEvents();
    """)
    session.actions.release()
    expected = [
        {"type": "mouseup"},
        {"type": "keyup"},
    ]
    events = [filter_dict(e, expected[0]) for e in get_events(session)]
    assert events == expected
