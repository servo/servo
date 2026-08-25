import pytest

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


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

    await subscribe_events(events=[CONTEXT_LOAD_EVENT])
    on_entry = wait_for_event(CONTEXT_LOAD_EVENT)
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
