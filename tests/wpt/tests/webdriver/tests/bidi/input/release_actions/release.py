import pytest
from webdriver.bidi.modules.script import ContextTarget

from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_release_no_actions_sends_no_events(
    bidi_session, top_context, load_static_test_page, get_focused_key_input
):
    await load_static_test_page(page="test_actions.html")
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
    events = await get_events(bidi_session, top_context["context"])

    assert len(keys["value"]) == 0
    assert len(events) == 0
