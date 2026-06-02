import pytest
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_dict, filter_supported_key_events
from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_release_mouse_sequence_resets_dblclick_state(
    bidi_session, new_tab, get_element, load_static_test_page
):
    """
    The actual behavior of the double click, specifically in the light of the `release_actions`
    is not clear in the spec at the moment: https://github.com/w3c/webdriver/issues/1772

    For now run this test in a new tab until it's clear if a cross-origin navigation
    should reset the input state map or not: https://github.com/w3c/webdriver/issues/1859
    """
    await load_static_test_page(page="test_actions.html", context=new_tab)
    reporter = await get_element("#outer", context=new_tab)

    actions = Actions()
    actions.add_pointer(pointer_type="mouse").pointer_move(
        x=0, y=0, origin=get_element_origin(reporter)
    ).pointer_down(button=0).pointer_up(button=0)
    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )

    await bidi_session.input.release_actions(context=new_tab["context"])

    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )
    events = await get_events(bidi_session, new_tab["context"])

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
