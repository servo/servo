import pytest

from ... import create_console_api_message, recursive_compare


@pytest.mark.asyncio
async def test_subscribe_one_user_context(bidi_session, subscribe_events, create_user_context, wait_for_events):
    user_context = await create_user_context()

    default_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context="default"
    )

    other_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=user_context
    )

    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

    with wait_for_events(["log.entryAdded"]) as waiter:
        await create_console_api_message(bidi_session, default_context, "text1")
        await create_console_api_message(bidi_session, other_context, "text2")
        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 1

        recursive_compare(
            {
                "text": "text2",
            },
            events[0][1],
        )


@pytest.mark.asyncio
async def test_subscribe_multiple_user_contexts(bidi_session, subscribe_events, wait_for_events, create_user_context):
    user_context = await create_user_context()

    default_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context="default"
    )

    other_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=user_context
    )

    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context, "default"])

    with wait_for_events(["log.entryAdded"]) as waiter:
        await create_console_api_message(bidi_session, default_context, "text1")
        await create_console_api_message(bidi_session, other_context, "text2")
        events = await waiter.get_events(lambda events : len(events) >= 2)
        assert len(events) == 2
