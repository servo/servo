import pytest
from webdriver.bidi.modules.script import ContextTarget

from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_release_no_actions_sends_no_events(
    bidi_session, top_context, test_actions_page_bidi, get_focused_key_input
):
    await test_actions_page_bidi()
    elem = await get_focused_key_input()

    await bidi_session.input.release_actions(context=top_context["context"])

    keys = await bidi_session.script.call_function(
        function_declaration="""(elem) => {
            return elem.value;
        }""",
        arguments=[elem],
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    events = await get_events(top_context["context"], bidi_session)

    assert len(keys["value"]) == 0
    assert len(events) == 0
