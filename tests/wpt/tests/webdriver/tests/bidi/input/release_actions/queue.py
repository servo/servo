import asyncio
import pytest

from webdriver.bidi.modules.input import Actions

from tests.support.helpers import filter_supported_key_events
from tests.support.keys import Keys

from .. import add_mouse_listeners, get_events, get_keys_value
from ... import recursive_compare


pytestmark = pytest.mark.asyncio


async def test_parallel_key(bidi_session, top_context, setup_key_test):
    actions_1 = Actions()
    actions_1.add_key().key_down("a").key_down(Keys.SHIFT)

    actions_2 = Actions()
    actions_2.add_key().key_down("b")

    # Run the first release actions in-between to check that it is queued for
    # sequential execution, and the state is reset before the 2nd action.
    actions_performed = [
        bidi_session.input.perform_actions(
            actions=actions_1, context=top_context["context"]
        ),
        bidi_session.input.release_actions(context=top_context["context"]),
        bidi_session.input.perform_actions(
            actions=actions_2, context=top_context["context"]
        ),
        bidi_session.input.release_actions(context=top_context["context"]),
    ]
    await asyncio.gather(*actions_performed)

    expected = [
        {"code": "KeyA", "key": "a", "type": "keydown"},
        {"code": "KeyA", "key": "a", "type": "keypress"},
        {"code": "ShiftLeft", "key": "Shift", "type": "keydown"},
        {"code": "ShiftLeft", "key": "Shift", "type": "keyup"},
        {"code": "KeyA", "key": "a", "type": "keyup"},
        {"code": "KeyB", "key": "b", "type": "keydown"},
        {"code": "KeyB", "key": "b", "type": "keypress"},
        {"code": "KeyB", "key": "b", "type": "keyup"},
    ]

    all_events = await get_events(bidi_session, top_context["context"])
    (key_events, expected) = filter_supported_key_events(all_events, expected)

    recursive_compare(expected, key_events)

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == "ab"


async def test_parallel_pointer(bidi_session, get_test_page, top_context):
    url = get_test_page()
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete")

    await add_mouse_listeners(bidi_session, top_context)

    point_1 = {"x": 5, "y": 10}
    point_2 = {"x": 10, "y": 20}

    actions_1 = Actions()
    (
        actions_1.add_pointer()
        .pointer_move(x=point_1["x"], y=point_1["y"])
        .pointer_down(button=0)
    )

    actions_2 = Actions()
    (
        actions_2.add_pointer()
        .pointer_move(x=point_2["x"], y=point_2["y"])
        .pointer_down(button=0)
    )

    # Run the first release actions in-between to check that it is queued for
    # sequential execution, and the state is reset before the 2nd action.
    actions_performed = [
        bidi_session.input.perform_actions(
            actions=actions_1, context=top_context["context"]
        ),
        bidi_session.input.release_actions(context=top_context["context"]),
        bidi_session.input.perform_actions(
            actions=actions_2, context=top_context["context"]
        ),
        bidi_session.input.release_actions(context=top_context["context"]),
    ]
    await asyncio.gather(*actions_performed)

    common_attributes = {
        "button": 0,
        "buttons": 0,
        "detail": 1,
        "isTrusted": True,
        "clientX": point_1["x"],
        "clientY": point_1["y"],
    }

    mouse_events = [
        {"type": "mousemove"},
        {"type": "mousedown", "buttons": 1},
        {"type": "mouseup"},
        {"type": "click"},
    ]

    # Expected events for the first action.
    expected_events_1 = [{**common_attributes, **event}
                         for event in mouse_events]

    # Expected events for the second action.
    common_attributes.update(
        {"clientX": point_2["x"], "clientY": point_2["y"]})
    expected_events_2 = [{**common_attributes, **event}
                         for event in mouse_events]

    events = await get_events(bidi_session, top_context["context"])

    assert events[:4] == expected_events_1
    assert events[4:] == expected_events_2
