import pytest

# The basic use case of subscribing globally for a single event
# is covered by tests for each event in the dedicated folders.


@pytest.mark.asyncio
async def test_subscribe_to_module(bidi_session, subscribe_events, wait_for_event, wait_for_future_safe):
    # Subscribe to all browsing context events
    await subscribe_events(events=["browsingContext"])

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, _):
        events.append(method)

    remove_listener_contextCreated = bidi_session.add_event_listener(
        "browsingContext.contextCreated", on_event
    )
    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    # Wait for the last event
    on_entry_added = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 3

    remove_listener_contextCreated()
    remove_listener_domContentLoaded()
    remove_listener_load()


@pytest.mark.asyncio
async def test_subscribe_to_one_event_and_then_to_module(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe
):
    # Subscribe to one event
    await subscribe_events(events=["browsingContext.contextCreated"])

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, data):
        events.append(method)

    remove_listener_contextCreated = bidi_session.add_event_listener(
        "browsingContext.contextCreated", on_event
    )

    on_entry_added = wait_for_event("browsingContext.contextCreated")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    assert "browsingContext.contextCreated" in events

    # Subscribe to all browsing context events
    await subscribe_events(events=["browsingContext"])

    # Clean up the event list
    events = []

    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    # Wait for the last event
    on_entry_added = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    # Make sure we didn't receive duplicates
    assert len(events) == 3

    remove_listener_contextCreated()
    remove_listener_domContentLoaded()
    remove_listener_load()


@pytest.mark.asyncio
async def test_subscribe_to_module_and_then_to_one_event_again(
    bidi_session, subscribe_events, wait_for_event, wait_for_future_safe
):
    # Subscribe to all browsing context events
    await subscribe_events(events=["browsingContext"])

    # Track all received browsing context events in the events array
    events = []

    async def on_event(method, data):
        events.append(method)

    remove_listener_contextCreated = bidi_session.add_event_listener(
        "browsingContext.contextCreated", on_event
    )
    remove_listener_domContentLoaded = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_load = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    # Wait for the last event
    on_entry_added = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 3

    # Subscribe to one event again
    await subscribe_events(events=["browsingContext.contextCreated"])

    # Clean up the event list
    events = []

    # Wait for the last event
    on_entry_added = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.create(type_hint="tab")
    await wait_for_future_safe(on_entry_added)

    # Make sure we didn't receive duplicates
    assert len(events) == 3

    remove_listener_contextCreated()
    remove_listener_domContentLoaded()
    remove_listener_load()
