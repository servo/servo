import pytest
from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_supported_key_events
from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_different_top_level_contexts(
    bidi_session, new_tab, top_context, test_actions_page_bidi, get_focused_key_input
):
    await test_actions_page_bidi()
    await get_focused_key_input()

    actions = Actions()
    actions.add_key().key_down("a")
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    # Reset so we only see the release events
    await bidi_session.script.evaluate(
        expression="resetEvents()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    # Release actions in another context
    await bidi_session.input.release_actions(context=new_tab["context"])

    events = await get_events(top_context["context"], bidi_session)
    assert len(events) == 0

    # Release actions in right context
    await bidi_session.input.release_actions(context=top_context["context"])

    expected = [
        {"code": "KeyA", "key": "a", "type": "keyup"},
    ]
    all_events = await get_events(top_context["context"], bidi_session)
    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected
