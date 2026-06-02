import pytest

from tests.classic.release_actions.support.refine import get_events
from tests.support.helpers import filter_dict


@pytest.mark.parametrize(
    "release_actions",
    [True, False],
    ids=["with release actions", "without release actions"],
)
def test_release_mouse_sequence_resets_dblclick_state(
    session, http_new_tab, test_actions_page, mouse_chain, release_actions
):
    """
    The actual behaviour of the double click, specifically in the light of the `release_actions`
    is not clear in the spec at the moment: https://github.com/w3c/webdriver/issues/1772

    For now run this test in a new tab until it's clear if a cross-origin navigation
    should reset the input state map or not: https://github.com/w3c/webdriver/issues/1859
    """
    reporter = session.find.css("#outer", all=False)

    mouse_chain.click(element=reporter).perform()

    if release_actions:
        session.actions.release()

    mouse_chain.perform()
    events = get_events(session)

    # The expected data here might vary between the vendors since the spec at the moment
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
