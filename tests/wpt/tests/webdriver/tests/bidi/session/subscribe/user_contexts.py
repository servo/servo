import pytest

from ... import create_console_api_message, recursive_compare

pytestmark = pytest.mark.asyncio


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


async def test_subscribe_default_user_context(bidi_session, subscribe_events, create_user_context, wait_for_events):
    user_context = await create_user_context()

    default_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context="default"
    )

    other_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=user_context
    )

    await subscribe_events(events=["log.entryAdded"], user_contexts=["default"])

    with wait_for_events(["log.entryAdded"]) as waiter:
        await create_console_api_message(bidi_session, default_context, "text1")
        await create_console_api_message(bidi_session, other_context, "text2")
        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 1

        recursive_compare(
            {
                "text": "text1",
            },
            events[0][1],
        )


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


async def test_buffered_event(
    bidi_session, subscribe_events, create_user_context, wait_for_events
):
    user_context = await create_user_context()

    new_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        await create_console_api_message(bidi_session, new_context, "text1")
        events = await waiter.get_events(lambda events: len(events) >= 0)

        # Make sure we didn't receive any events
        assert len(events) == 0

        # Subscribe to user context and make sure we received the buffered event.
        await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": "text1",
            },
            events[0][1],
        )


async def test_subscribe_to_user_context_and_then_globally(
    bidi_session, subscribe_events, create_user_context, new_tab, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to a specific user context
    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in another user context
        event_expected_text = await create_console_api_message(
            bidi_session, new_tab, "text1"
        )

        events = await waiter.get_events(lambda events: len(events) >= 0)

        assert len(events) == 0

        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text2"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

        # Subscribe for log events globally
        await subscribe_events(events=["log.entryAdded"])

        events = await waiter.get_events(lambda events: len(events) >= 2)

        # Check that we received the buffered event
        assert len(events) == 2
        recursive_compare(
            {
                "text": event_expected_text,
            },
            events[1][1],
        )

        # Trigger again events in each context
        expected_text = await create_console_api_message(bidi_session, new_tab, "text3")
        events = await waiter.get_events(lambda events: len(events) >= 3)

        assert len(events) == 3
        recursive_compare(
            {
                "text": expected_text,
            },
            events[2][1],
        )

        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text4"
        )
        events = await waiter.get_events(lambda events: len(events) >= 4)

        assert len(events) == 4
        recursive_compare(
            {
                "text": expected_text,
            },
            events[3][1],
        )


async def test_subscribe_to_user_context_and_then_to_browsing_context(
    bidi_session, subscribe_events, create_user_context, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to a specific user context
    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the observed context
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Subscribe for log events to the browsing context
    # in the observed user context.
    await subscribe_events(
        events=["log.entryAdded"], contexts=[new_context_in_user_context["context"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger again the event
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text2"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Create a new context in the default user context
    new_context_in_default_user_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    # Subscribe for log events to the browsing context
    # in the default user context.
    await subscribe_events(
        events=["log.entryAdded"],
        contexts=[new_context_in_default_user_context["context"]],
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger again the event
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_default_user_context, "text3"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )


async def test_subscribe_globally_and_then_to_user_context(
    bidi_session, subscribe_events, create_user_context, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events globally
    await subscribe_events(events=["log.entryAdded"])

    with wait_for_events(["log.entryAdded"]) as waiter:
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Subscribe for log events to a specific user context
    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger the event again.
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text2"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )


async def test_subscribe_to_browsing_context_and_then_to_user_context(
    bidi_session, subscribe_events, create_user_context, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to browsing context
    await subscribe_events(
        events=["log.entryAdded"], contexts=[new_context_in_user_context["context"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Subscribe for log events to a specific user context
    await subscribe_events(events=["log.entryAdded"], user_contexts=[user_context])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger the event again.
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context, "text2"
        )
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

        # Create a new context in the targeted user context
        new_context_in_user_context_2 = await bidi_session.browsing_context.create(
            user_context=user_context, type_hint="tab"
        )

        # Trigger again the event
        expected_text = await create_console_api_message(
            bidi_session, new_context_in_user_context_2, "text3"
        )
        events = await waiter.get_events(lambda events: len(events) >= 2)

        assert len(events) == 2
        recursive_compare(
            {
                "text": expected_text,
            },
            events[1][1],
        )
