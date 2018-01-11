# META: timeout=long

import pytest

from tests.actions.support.refine import filter_dict, get_events
from tests.actions.support.keys import Keys


# Using local fixtures because we want to start a new session between
# each test, otherwise the clicks in each test interfere with each other.
@pytest.fixture(autouse=True)
def release_actions(mod_click_session, request):
    request.addfinalizer(mod_click_session.actions.release)


@pytest.fixture
def mod_click_session(new_session, url, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({})}})
    session.url = url("/webdriver/tests/actions/support/test_actions_wdspec.html")

    return session


@pytest.fixture
def key_chain(mod_click_session):
    return mod_click_session.actions.sequence("key", "keyboard_id")


@pytest.fixture
def mouse_chain(mod_click_session):
    return mod_click_session.actions.sequence(
        "pointer",
        "pointer_id",
        {"pointerType": "mouse"})


@pytest.mark.parametrize("modifier, prop", [
   (Keys.ALT, "altKey"),
   (Keys.R_ALT, "altKey"),
   (Keys.META, "metaKey"),
   (Keys.R_META, "metaKey"),
   (Keys.SHIFT, "shiftKey"),
   (Keys.R_SHIFT, "shiftKey"),
])
def test_modifier_click(mod_click_session,
                       key_chain,
                       mouse_chain,
                       modifier,
                       prop):
    key_chain \
        .pause(200) \
        .key_down(modifier) \
        .pause(200) \
        .key_up(modifier)
    outer = mod_click_session.find.css("#outer", all=False)
    mouse_chain.click(element=outer)
    mod_click_session.actions.perform([key_chain.dict, mouse_chain.dict])
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
    filtered_events = [filter_dict(e, expected[0]) for e in get_events(mod_click_session)]
    assert expected == filtered_events


def test_many_modifiers_click(mod_click_session, key_chain, mouse_chain):
    outer = mod_click_session.find.css("#outer", all=False)
    key_chain \
        .pause(0) \
        .key_down(Keys.CONTROL) \
        .key_down(Keys.SHIFT) \
        .pause(0) \
        .key_up(Keys.CONTROL) \
        .key_up(Keys.SHIFT)
    mouse_chain \
        .pointer_move(0, 0, origin=outer) \
        .pause(0) \
        .pointer_down() \
        .pointer_up() \
        .pause(0) \
        .pause(0) \
        .pointer_down()
    mod_click_session.actions.perform([key_chain.dict, mouse_chain.dict])
    expected = [
        {"type": "mousemove"},
        # shift and ctrl presses
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
        e["ctrlKey"] = True
    events = [filter_dict(e, expected[0]) for e in get_events(mod_click_session)]
    assert events == expected
