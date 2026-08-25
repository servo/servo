import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

MESSAGE_EVENT = "script.message"


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    wait_for_event,
    wait_for_future_safe,
    user_context,
):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )

    await subscribe_events(events=[MESSAGE_EVENT])
    on_entry = wait_for_event(MESSAGE_EVENT)
    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event["source"]
    assert event["source"]["userContext"] == user_context_id
