import pytest
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_dict, filter_supported_key_events
from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_release_char_sequence_sends_keyup_events_in_reverse(
    bidi_session, top_context, load_static_test_page, get_focused_key_input
):
    await load_static_test_page(page="test_actions.html")
    await get_focused_key_input()

    actions = Actions()
    actions.add_key().key_down("a").key_down("b")
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    # Reset so we only see the release events
    await bidi_session.script.evaluate(
        expression="resetEvents()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    await bidi_session.input.release_actions(context=top_context["context"])
    expected = [
        {"code": "KeyB", "key": "b", "type": "keyup"},
        {"code": "KeyA", "key": "a", "type": "keyup"},
    ]
    all_events = await get_events(bidi_session, top_context["context"])
    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected


@pytest.mark.parametrize(
    "release_actions",
    [True, False],
    ids=["with release actions", "without release actions"],
)
async def test_release_mouse_sequence_resets_dblclick_state(
    bidi_session,
    top_context,
    get_element,
    load_static_test_page,
    release_actions
):
    await load_static_test_page(page="test_actions.html")
    reporter = await get_element("#outer")

    actions = Actions()
    actions.add_pointer(pointer_type="mouse").pointer_move(
        x=0, y=0, origin=get_element_origin(reporter)
    ).pointer_down(button=0).pointer_up(button=0)
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    if release_actions:
        await bidi_session.input.release_actions(context=top_context["context"])

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    events = await get_events(bidi_session, top_context["context"])

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
