import pytest

from .. import PAGE_EMPTY_HTML, BEFORE_REQUEST_SENT_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    subscribe_events,
    create_user_context,
    wait_for_event,
    wait_for_future_safe,
    url,
    user_context,
):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )

    await subscribe_events(events=[BEFORE_REQUEST_SENT_EVENT])
    on_entry = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=url(PAGE_EMPTY_HTML), wait="complete"
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
