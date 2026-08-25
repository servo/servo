import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

HISTORY_UPDATED_EVENT = "browsingContext.historyUpdated"


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    inline,
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

    await bidi_session.browsing_context.navigate(
        context=context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    await subscribe_events(events=[HISTORY_UPDATED_EVENT])
    on_entry = wait_for_event(HISTORY_UPDATED_EVENT)
    await bidi_session.script.evaluate(
        expression="history.pushState(null, null, '/some-path')",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
