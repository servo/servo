import pytest
from webdriver.bidi.modules.script import ContextTarget

from . import assert_console_entry

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "sandbox_name",
    ["", "sandbox_1"],
    ids=["default realm", "sandbox"],
)
async def test_realm(bidi_session, subscribe_events, top_context, wait_for_event, wait_for_future_safe, sandbox_name):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = "foo"
    result = await bidi_session.script.evaluate(
        raw_result=True,
        expression=f"console.log('{expected_text}')",
        await_promise=False,
        target=ContextTarget(top_context["context"], sandbox=sandbox_name),
    )
    event_data = await wait_for_future_safe(on_entry_added)

    assert_console_entry(
        event_data,
        text=expected_text,
        context=top_context["context"],
        realm=result["realm"],
    )
