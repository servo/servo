import pytest

pytestmark = pytest.mark.asyncio

REALM_CREATED_EVENT = "script.realmCreated"


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session, subscribe_events, create_user_context, wait_for_bidi_events, user_context
):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    await subscribe_events(events=[REALM_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(REALM_CREATED_EVENT, on_event)
    await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )
    await wait_for_bidi_events(events, 1, equal_check=False)
    remove_listener()

    realm_event = events[-1]
    assert "userContext" in realm_event
    assert realm_event["userContext"] == user_context_id
