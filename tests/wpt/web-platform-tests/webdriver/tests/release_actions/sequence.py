# META: timeout=long

from tests.release_actions.support.refine import get_events, get_keys
from tests.support.helpers import filter_dict


def test_release_no_actions_sends_no_events(session, key_reporter):
    session.actions.release()
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
    if len(events) > 0 and events[0]["code"] is None:
        # Remove 'code' entry if browser doesn't support it
        expected = [filter_dict(e, {"key": "", "type": ""}) for e in expected]
        events = [filter_dict(e, expected[0]) for e in events]
    assert events == expected


def test_release_mouse_sequence_resets_dblclick_state(session,
                                                      test_actions_page,
                                                      mouse_chain):
    reporter = session.find.css("#outer", all=False)

    mouse_chain \
        .click(element=reporter) \
        .perform()
    session.actions.release()
    mouse_chain \
        .perform()
    events = get_events(session)

    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


def test_no_release_mouse_sequence_keeps_dblclick_state(session,
                                                        test_actions_page,
                                                        mouse_chain):
    reporter = session.find.css("#outer", all=False)

    mouse_chain \
        .click(element=reporter) \
        .perform()
    mouse_chain \
        .perform()
    events = get_events(session)

    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "dblclick", "button": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]
