import pytest

from tests.classic.release_actions.support.refine import get_events, get_keys
from tests.support.helpers import filter_supported_key_events


def test_release_no_actions_sends_no_events(session, key_reporter):
    session.actions.release()
    assert len(get_keys(key_reporter)) == 0
    assert len(get_events(session)) == 0


def test_release_char_sequence_sends_keyup_events_in_reverse(
    session, key_reporter, key_chain
):
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
    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected
