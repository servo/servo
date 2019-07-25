import pytest

from tests.perform_actions.support.keys import Keys
from tests.perform_actions.support.refine import filter_dict, get_events


@pytest.mark.parametrize("modifier, prop", [
    (Keys.CONTROL, "ctrlKey"),
    (Keys.R_CONTROL, "ctrlKey"),
])
def test_control_click(session, test_actions_page, key_chain, mouse_chain, modifier, prop):
    os = session.capabilities["platformName"]
    key_chain \
        .pause(0) \
        .key_down(modifier) \
        .pause(200) \
        .key_up(modifier)
    outer = session.find.css("#outer", all=False)
    mouse_chain.click(element=outer)
    session.actions.perform([key_chain.dict, mouse_chain.dict])
    if os != "mac":
        expected = [
            {"type": "mousemove"},
            {"type": "mousedown"},
            {"type": "mouseup"},
            {"type": "click"},
        ]
    else:
        expected = [
            {"type": "mousemove"},
            {"type": "mousedown"},
            {"type": "contextmenu"},
            {"type": "mouseup"},
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
    # The context menu stays visible during subsequent tests so let's not
    # display it in the first place.
    session.execute_script("""
        var keyReporter = document.getElementById("keys");
        document.addEventListener("contextmenu", function(e) {
          e.preventDefault();
        });
    """)
    key_chain \
        .pause(0) \
        .key_down(Keys.CONTROL)
    mouse_chain \
        .pointer_move(0, 0, origin=key_reporter) \
        .pointer_down()
    session.actions.perform([key_chain.dict, mouse_chain.dict])
    session.execute_script("""
        var keyReporter = document.getElementById("keys");
        keyReporter.addEventListener("mousedown", recordPointerEvent);
        keyReporter.addEventListener("mouseup", recordPointerEvent);
        resetEvents();
    """)
    session.actions.release()
    expected = [
        {"type": "mouseup"},
        {"type": "keyup"},
    ]
    events = [filter_dict(e, expected[0]) for e in get_events(session)]
    assert events == expected
