import pytest

from ... import create_console_api_message, recursive_compare

pytestmark = pytest.mark.asyncio


async def test_unsubscribe_with_subscription_id(
    bidi_session, top_context, wait_for_events
):
    result = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    # Unsubscribe from log events in one of the contexts
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event
        await create_console_api_message(bidi_session, top_context, "text1")
        events = await waiter.get_events(lambda events: len(events) >= 0)
        assert len(events) == 0


async def test_unsubscribe_with_multiple_subscription_ids(
    bidi_session, new_tab, inline
):
    # Subscribe to multiple events
    result_1 = await bidi_session.session.subscribe(
        events=["browsingContext.domContentLoaded"]
    )
    result_2 = await bidi_session.session.subscribe(events=["browsingContext.load"])

    # Unsubscribe from both events with subscription ids
    await bidi_session.session.unsubscribe(
        subscriptions=[result_1["subscription"], result_2["subscription"]]
    )

    # Track all received events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener_1 = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_2 = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    # Trigger browsingContext events
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    # Make sure that we didn't receive any events
    assert len(events) == 0

    remove_listener_1()
    remove_listener_2()


async def test_unsubscribe_from_one_of_the_context(
    bidi_session, top_context, new_tab, wait_for_events
):
    # Subscribe for log events to multiple contexts
    result_1 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )
    result_2 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )
    # Unsubscribe from log events in one of the subscriptions
    await bidi_session.session.unsubscribe(subscriptions=[result_1["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the unsubscribed context
        await create_console_api_message(bidi_session, top_context, "text1")
        events = await waiter.get_events(lambda events: len(events) >= 0)
        assert len(events) == 0

        # Trigger another console event in the still observed context
        expected_text = await create_console_api_message(bidi_session, new_tab, "text2")
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    await bidi_session.session.unsubscribe(subscriptions=[result_2["subscription"]])


async def test_unsubscribe_from_closed_context(bidi_session):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    result = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_context["context"]]
    )

    # Close the context.
    await bidi_session.browsing_context.close(context=new_context["context"])

    # Unsubscribe using subscription id to make sure that there is no error.
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])


async def test_subscribe_twice_to_context_and_unsubscribe_once(
    bidi_session, new_tab, wait_for_events
):
    # Subscribe twice to a context
    result_1 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )
    result_2 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )

    # Unsubscribe once
    await bidi_session.session.unsubscribe(subscriptions=[result_1["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
        expected_text = await create_console_api_message(bidi_session, new_tab, "text")
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    await bidi_session.session.unsubscribe(subscriptions=[result_2["subscription"]])


async def test_subscribe_twice_to_globally_and_unsubscribe_once(
    bidi_session, new_tab, wait_for_events
):
    # Subscribe twice globally
    result_1 = await bidi_session.session.subscribe(events=["log.entryAdded"])
    result_2 = await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Unsubscribe once
    await bidi_session.session.unsubscribe(subscriptions=[result_1["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event
        expected_text = await create_console_api_message(bidi_session, new_tab, "text")
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    await bidi_session.session.unsubscribe(subscriptions=[result_2["subscription"]])


async def test_unsubscribe_partially_from_one_event(bidi_session, top_context, inline):
    # Subscribe to multiple events at once
    result = await bidi_session.session.subscribe(
        events=["browsingContext.domContentLoaded", "browsingContext.load"]
    )
    # Unsubscribe from one event
    await bidi_session.session.unsubscribe(events=["browsingContext.domContentLoaded"])

    # Track all received browsingContext.domContentLoaded and browsingContext.load events in the events arrays
    events_domContentLoaded = []
    events_load = []

    async def on_domContentLoaded_event(method, data):
        events_domContentLoaded.append(data)

    async def on_load_event(method, data):
        events_load.append(data)

    remove_listener_1 = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_domContentLoaded_event
    )
    remove_listener_2 = bidi_session.add_event_listener(
        "browsingContext.load", on_load_event
    )

    # Trigger browsingContext events in the unsubscribed context
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    # Make sure that we received "browsingContext.load" event
    assert len(events_domContentLoaded) == 0
    assert len(events_load) == 1

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    # Trigger browsingContext events in the unsubscribed context
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    # Make sure that there're no new events
    assert len(events_domContentLoaded) == 0
    assert len(events_load) == 1

    remove_listener_1()
    remove_listener_2()


async def test_unsubscribe_with_event_and_subscriptions(bidi_session, new_tab, inline):
    result = await bidi_session.session.subscribe(events=["browsingContext"])

    # Provide both `events` and `subscriptions` arguments
    await bidi_session.session.unsubscribe(
        events=["browsingContext.domContentLoaded"],
        subscriptions=[result["subscription"]],
    )

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, _):
        events.append(method)

    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div></div>"), wait="complete"
    )

    # Make sure we didn't receive any events. Which would indicate that
    # `subscriptions` argument took precedent over `events`.
    assert len(events) == 0

    remove_listener_domContentLoaded()
    remove_listener_load()


async def test_unsubscribe_globally_with_one_context_subscription(
    bidi_session, new_tab, wait_for_events
):
    # Subscribe for log events to one context
    result_one_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )
    # Subscribe globally
    result_globally = await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Unsubscribe from log events globally
    await bidi_session.session.unsubscribe(
        subscriptions=[result_globally["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
        expected_text = await create_console_api_message(bidi_session, new_tab, "text")
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_one_context["subscription"]]
    )


async def test_unsubscribe_from_one_context_with_global_subscription(
    bidi_session, new_tab, wait_for_events
):
    # Subscribe for log events to one context
    result_one_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )
    # Subscribe globally
    result_globally = await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Unsubscribe from log events from one context
    await bidi_session.session.unsubscribe(
        subscriptions=[result_one_context["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
        expected_text = await create_console_api_message(bidi_session, new_tab, "text")
        events = await waiter.get_events(lambda events: len(events) >= 1)

        assert len(events) == 1
        recursive_compare(
            {
                "text": expected_text,
            },
            events[0][1],
        )

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_globally["subscription"]]
    )


async def test_unsubscribe_globally_with_user_context_subscription(
    bidi_session, wait_for_events, create_user_context
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Subscribe for log events to user context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )
    # Subscribe globally
    result_globally = await bidi_session.session.subscribe(events=["log.entryAdded"])
    # Unsubscribe from log events globally
    await bidi_session.session.unsubscribe(
        subscriptions=[result_globally["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_user_context["subscription"]]
    )


async def test_unsubscribe_globally_with_user_context_and_context_subscription(
    bidi_session, wait_for_events, create_user_context
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_context_in_user_context["context"]]
    )
    # Subscribe for log events to user context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )
    # Subscribe globally
    result_globally = await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Unsubscribe from log events globally
    await bidi_session.session.unsubscribe(
        subscriptions=[result_globally["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_user_context["subscription"]]
    )


async def test_unsubscribe_from_user_context_with_global_subscription(
    bidi_session, wait_for_events, create_user_context
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to user context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )
    # Subscribe globally
    result_globally = await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Unsubscribe from log events in user context
    await bidi_session.session.unsubscribe(
        subscriptions=[result_user_context["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_globally["subscription"]]
    )


async def test_unsubscribe_from_user_context(
    bidi_session, create_user_context, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    result = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )

    # Unsubscribe from log events in the user context
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event
        await create_console_api_message(
            bidi_session, new_context_in_user_context, "text1"
        )
        events = await waiter.get_events(lambda events: len(events) >= 0)
        assert len(events) == 0


async def test_unsubscribe_from_user_context_twice(
    bidi_session, create_user_context, wait_for_events
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe to log events in the user context twice
    result_1 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )
    result_2 = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )

    # Unsubscribe from log events in the user context once
    await bidi_session.session.unsubscribe(subscriptions=[result_1["subscription"]])

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(subscriptions=[result_2["subscription"]])


async def test_unsubscribe_from_user_context_with_context_subscription(
    bidi_session, wait_for_events, create_user_context
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to context
    result_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_context_in_user_context["context"]]
    )
    # Subscribe for log events to user context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )

    # Unsubscribe from log events from user context
    await bidi_session.session.unsubscribe(
        subscriptions=[result_user_context["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_context["subscription"]]
    )


async def test_unsubscribe_from_context_with_user_context_subscription(
    bidi_session, wait_for_events, create_user_context
):
    user_context = await create_user_context()
    new_context_in_user_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Subscribe for log events to context
    result_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[new_context_in_user_context["context"]]
    )
    # Subscribe for log events to user context
    result_user_context = await bidi_session.session.subscribe(
        events=["log.entryAdded"], user_contexts=[user_context]
    )

    # Unsubscribe from log events from context
    await bidi_session.session.unsubscribe(
        subscriptions=[result_context["subscription"]]
    )

    with wait_for_events(["log.entryAdded"]) as waiter:
        # Trigger console event in the still observed context
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

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(
        subscriptions=[result_user_context["subscription"]]
    )
