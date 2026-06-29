import pytest

pytestmark = pytest.mark.asyncio

FRAGMENT_NAVIGATED_EVENT = "browsingContext.fragmentNavigated"


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

    page_url = inline("<div id='target'>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=page_url, wait="complete"
    )
    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])
    on_entry = wait_for_event(FRAGMENT_NAVIGATED_EVENT)
    await bidi_session.browsing_context.navigate(
        context=context["context"], url=page_url + "#target", wait="complete"
    )
    event = await wait_for_future_safe(on_entry)
    assert "userContext" in event
    assert event["userContext"] == user_context_id
