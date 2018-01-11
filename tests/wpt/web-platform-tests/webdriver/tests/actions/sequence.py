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
