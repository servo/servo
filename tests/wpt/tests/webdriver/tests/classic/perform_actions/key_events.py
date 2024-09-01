# META: timeout=long
import copy
from collections import defaultdict

import pytest

from tests.classic.perform_actions.support.refine import get_events, get_keys
from tests.support.helpers import filter_dict, filter_supported_key_events
from tests.support.keys import ALL_EVENTS, ALTERNATIVE_KEY_NAMES, Keys


def get_key_events(session):
    """Return list of key events. Filters out non-key events to prevent noise
    from OS mouse events."""
    all_events = get_events(session)
    return [event for event in all_events if event["type"].startswith("key")]


def test_keyup_only_sends_no_events(session, key_reporter, key_chain):
    key_chain.key_up("a").perform()

    assert len(get_keys(key_reporter)) == 0
    assert len(get_key_events(session)) == 0

    session.actions.release()
    assert len(get_keys(key_reporter)) == 0
    assert len(get_key_events(session)) == 0


@pytest.mark.parametrize("key, event", [
    (Keys.ALT, "ALT"),
    (Keys.CONTROL, "CONTROL"),
    (Keys.META, "META"),
    (Keys.SHIFT, "SHIFT"),
    (Keys.R_ALT, "R_ALT"),
    (Keys.R_CONTROL, "R_CONTROL"),
    (Keys.R_META, "R_META"),
    (Keys.R_SHIFT, "R_SHIFT"),
])
def test_modifier_key_sends_correct_events(session, key_reporter, key_chain, key, event):
    code = ALL_EVENTS[event]["code"]
    value = ALL_EVENTS[event]["key"]

    if session.capabilities["browserName"] == "internet explorer":
        key_reporter.click()
        session.execute_script("resetEvents();")
    key_chain \
        .key_down(key) \
        .key_up(key) \
        .perform()
    all_events = get_key_events(session)

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    assert len(get_keys(key_reporter)) == 0


@pytest.mark.parametrize("key,event", [
    (Keys.ESCAPE, "ESCAPE"),
    (Keys.RIGHT, "RIGHT"),
])
def test_non_printable_key_sends_events(session, key_reporter, key_chain, key, event):
    code = ALL_EVENTS[event]["code"]
    value = ALL_EVENTS[event]["key"]

    key_chain \
        .key_down(key) \
        .key_up(key) \
        .perform()
    all_events = get_key_events(session)

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keypress"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    # Make a copy for alternate key property values
    # Note: only keydown and keyup are affected by alternate key names
    alt_expected = copy.deepcopy(expected)
    if event in ALTERNATIVE_KEY_NAMES:
        alt_expected[0]["key"] = ALTERNATIVE_KEY_NAMES[event]
        alt_expected[2]["key"] = ALTERNATIVE_KEY_NAMES[event]

    (_, expected) = filter_supported_key_events(all_events, expected)
    (events, alt_expected) = filter_supported_key_events(all_events, alt_expected)
    if len(events) == 2:
        # most browsers don't send a keypress for non-printable keys
        assert events == [expected[0], expected[2]] or events == [alt_expected[0], alt_expected[2]]
    else:
        assert events == expected or events == alt_expected

    assert len(get_keys(key_reporter)) == 0


@pytest.mark.parametrize("value,code", [
    (u"a", "KeyA",),
    ("a", "KeyA",),
    (u"\"", "Quote"),
    (u",", "Comma"),
    (u"\u00E0", ""),
    (u"\u0416", ""),
    (u"@", "Digit2"),
    (u"\u2603", ""),
    (u"\uF6C2", ""),  # PUA
])
def test_printable_key_sends_correct_events(session, key_reporter, key_chain, value, code):
    key_chain \
        .key_down(value) \
        .key_up(value) \
        .perform()
    all_events = get_key_events(session)

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keypress"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    assert get_keys(key_reporter) == value


def test_sequence_of_keydown_printable_keys_sends_events(session, key_reporter, key_chain):
    key_chain \
        .key_down("a") \
        .key_down("b") \
        .perform()
    all_events = get_key_events(session)

    expected = [
        {"code": "KeyA", "key": "a", "type": "keydown"},
        {"code": "KeyA", "key": "a", "type": "keypress"},
        {"code": "KeyB", "key": "b", "type": "keydown"},
        {"code": "KeyB", "key": "b", "type": "keypress"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    assert get_keys(key_reporter) == "ab"


def test_sequence_of_keydown_printable_characters_sends_events(session, key_reporter, key_chain):
    key_chain.send_keys("ef").perform()
    all_events = get_key_events(session)

    expected = [
        {"code": "KeyE", "key": "e", "type": "keydown"},
        {"code": "KeyE", "key": "e", "type": "keypress"},
        {"code": "KeyE", "key": "e", "type": "keyup"},
        {"code": "KeyF", "key": "f", "type": "keydown"},
        {"code": "KeyF", "key": "f", "type": "keypress"},
        {"code": "KeyF", "key": "f", "type": "keyup"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    assert get_keys(key_reporter) == "ef"


@pytest.mark.parametrize("name,expected", ALL_EVENTS.items())
def test_special_key_sends_keydown(session, key_reporter, key_chain, name, expected):
    if name.startswith("F"):
        # Prevent default behavior for F1, etc., but only after keydown
        # bubbles up to body. (Otherwise activated browser menus/functions
        # may interfere with subsequent tests.)
        session.execute_script("""
            document.body.addEventListener("keydown",
                    function(e) { e.preventDefault() });
        """)
    if session.capabilities["browserName"] == "internet explorer":
        key_reporter.click()
        session.execute_script("resetEvents();")
    key_chain.key_down(getattr(Keys, name)).perform()

    # only interested in keydown
    first_event = get_key_events(session)[0]
    # make a copy so we can throw out irrelevant keys and compare to events
    expected = dict(expected)

    del expected["value"]

    # make another copy for alternative key names
    alt_expected = copy.deepcopy(expected)
    if name in ALTERNATIVE_KEY_NAMES:
        alt_expected["key"] = ALTERNATIVE_KEY_NAMES[name]

    # check and remove keys that aren't in expected
    assert first_event["type"] == "keydown"
    assert first_event["repeat"] is False
    first_event = filter_dict(first_event, expected)
    if first_event["code"] is None:
        del first_event["code"]
        del expected["code"]
        del alt_expected["code"]
    assert first_event == expected or first_event == alt_expected
    # only printable characters should be recorded in input field
    entered_keys = get_keys(key_reporter)
    if len(expected["key"]) == 1:
        assert entered_keys == expected["key"]
    else:
        assert len(entered_keys) == 0


def test_space_char_equals_pua(session, key_reporter, key_chain):
    key_chain \
        .key_down(Keys.SPACE) \
        .key_up(Keys.SPACE) \
        .key_down(" ") \
        .key_up(" ") \
        .perform()
    all_events = get_key_events(session)
    by_type = defaultdict(list)
    for event in all_events:
        by_type[event["type"]].append(event)

    for event_type in by_type:
        events = by_type[event_type]
        assert len(events) == 2
        assert events[0] == events[1]
