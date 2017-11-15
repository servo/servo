# META: timeout=long

from tests.actions.support.refine import get_keys, filter_dict, get_events
from tests.actions.support.keys import Keys


def test_no_actions_send_no_events(session, key_reporter, key_chain):
    key_chain.perform()
    assert len(get_keys(key_reporter)) == 0
    assert len(get_events(session)) == 0


def test_release_char_sequence_sends_keyup_events_in_reverse(session,
                                                             key_reporter,
                                                             key_chain):
    key_chain \
        .key_down("a") \
        .key_down("b") \
        .perform()
    # reset so we only see the release events
    session.execute_script("resetEvents();")
    session.actions.release()
    expected = [
        {"code": "KeyB", "key": "b", "type": "keyup"},
        {"code": "KeyA", "key": "a", "type": "keyup"},
    ]
    all_events = get_events(session)
    events = [filter_dict(e, expected[0]) for e in all_events]
    if len(events) > 0 and events[0]["code"] == None:
        # Remove 'code' entry if browser doesn't support it
        expected = [filter_dict(e, {"key": "", "type": ""}) for e in expected]
        events = [filter_dict(e, expected[0]) for e in events]
    assert events == expected


def test_release_no_actions_sends_no_events(session, key_reporter):
    session.actions.release()
    assert len(get_keys(key_reporter)) == 0
    assert len(get_events(session)) == 0


def test_many_modifiers_click(session, test_actions_page, key_chain, mouse_chain):
    outer = session.find.css("#outer", all=False)
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
    session.actions.perform([key_chain.dict, mouse_chain.dict])
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
    events = [filter_dict(e, expected[0]) for e in get_events(session)]
    assert events == expected
