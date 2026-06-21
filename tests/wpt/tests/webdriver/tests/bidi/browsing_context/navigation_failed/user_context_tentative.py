import pytest

pytestmark = pytest.mark.asyncio

NAVIGATION_FAILED_EVENT = "browsingContext.navigationFailed"


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    inline,
    iframe,
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

    await subscribe_events(events=[NAVIGATION_FAILED_EVENT])
    on_entry = wait_for_event(NAVIGATION_FAILED_EVENT)
    page_url = inline(
        iframe("<div>foo</div>", domain="alt"),
        parameters={"pipe": "header(Content-Security-Policy, default-src 'self')"},
    )
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=page_url, wait="none"
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
