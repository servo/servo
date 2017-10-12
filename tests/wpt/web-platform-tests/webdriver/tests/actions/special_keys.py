# META: timeout=long

import pytest
from tests.actions.support.keys import ALL_EVENTS, Keys
from tests.actions.support.refine import filter_dict, get_keys, get_events


@pytest.mark.parametrize("name,expected", ALL_EVENTS.items())
def test_webdriver_special_key_sends_keydown(session,
                                             key_reporter,
                                             key_chain,
                                             name,
                                             expected):
    if name.startswith("F"):
        # Prevent default behavior for F1, etc., but only after keydown
        # bubbles up to body. (Otherwise activated browser menus/functions
        # may interfere with subsequent tests.)
        session.execute_script("""
            document.body.addEventListener("keydown",
                    (e) => e.preventDefault());
        """)
    key_chain.key_down(getattr(Keys, name)).perform()
    # only interested in keydown
    first_event = get_events(session)[0]
    # make a copy so we can throw out irrelevant keys and compare to events
    expected = dict(expected)

    del expected["value"]
    # check and remove keys that aren't in expected
    assert first_event["type"] == "keydown"
    assert first_event["repeat"] == False
    first_event = filter_dict(first_event, expected)
    assert first_event == expected
    # only printable characters should be recorded in input field
    entered_keys = get_keys(key_reporter)
    if len(expected["key"]) == 1:
        assert entered_keys == expected["key"]
    else:
        assert len(entered_keys) == 0
