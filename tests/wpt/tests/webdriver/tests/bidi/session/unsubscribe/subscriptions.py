import pytest

from ... import create_console_api_message, recursive_compare

pytestmark = pytest.mark.asyncio


async def test_unsubscribe_with_subscription_id(bidi_session, top_context):
    result = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    # Unsubscribe from log events in one of the contexts
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event
    await create_console_api_message(bidi_session, top_context, "text1")
    assert len(events) == 0

    remove_listener()


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
    bidi_session, top_context, new_tab, wait_for_event, wait_for_future_safe
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

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the unsubscribed context
    await create_console_api_message(bidi_session, top_context, "text1")
    assert len(events) == 0

    # Trigger another console event in the still observed context
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, new_tab, "text2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    recursive_compare(
        {
            "text": expected_text,
        },
        events[0],
    )

    remove_listener()
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


async def test_unsubscribe_partially_from_one_context(
    bidi_session, top_context, new_tab, wait_for_event, wait_for_future_safe
):
    # Subscribe for log events to multiple contexts at once
    result = await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"], new_tab["context"]]
    )
    # Unsubscribe from log events in one of the contexts
    await bidi_session.session.unsubscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the unsubscribed context
    await create_console_api_message(bidi_session, top_context, "text1")
    assert len(events) == 0

    # Trigger another console event in the still observed context
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, new_tab, "text2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    recursive_compare(
        {
            "text": expected_text,
        },
        events[0],
    )

    # Unsubscribe from subscription.
    await bidi_session.session.unsubscribe(subscriptions=[result["subscription"]])

    # Trigger another console event
    await create_console_api_message(bidi_session, new_tab, "text2")

    # Make sure that there're no new events
    assert len(events) == 1

    remove_listener()


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
