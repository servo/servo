import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


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

    await subscribe_events(events=["log.entryAdded"])
    on_entry = wait_for_event("log.entryAdded")
    await bidi_session.script.evaluate(
        expression="console.log('test')",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event["source"]
    assert event["source"]["userContext"] == user_context_id
