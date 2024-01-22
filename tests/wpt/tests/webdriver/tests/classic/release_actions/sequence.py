import pytest

from tests.classic.release_actions.support.refine import get_events, get_keys
from tests.support.helpers import filter_dict, filter_supported_key_events


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
    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected


@pytest.mark.parametrize(
    "release_actions",
    [True, False],
    ids=["with release actions", "without release actions"],
)
def test_release_mouse_sequence_resets_dblclick_state(session,
                                                      test_actions_page,
                                                      mouse_chain,
                                                      release_actions):
    reporter = session.find.css("#outer", all=False)

    mouse_chain \
        .click(element=reporter) \
        .perform()

    if release_actions:
        session.actions.release()

    mouse_chain \
        .perform()
    events = get_events(session)

    # The expeced data here might vary between the vendors since the spec at the moment
    # is not clear on how the double/triple click should be tracked. It should be
    # clarified in the scope of https://github.com/w3c/webdriver/issues/1772.
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
