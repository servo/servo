import asyncio

import pytest

from . import assert_base_entry, create_log


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_subscribe_twice(bidi_session, new_tab, wait_for_event, wait_for_future_safe, log_type):
    # Subscribe to log.entryAdded twice and check that events are received once.
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Check for a ConsoleLogEntry.
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_log(bidi_session, new_tab, log_type, "text1")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    assert_base_entry(events[0], text=expected_text)

    # Wait for some time and check the events array again
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_subscribe_unsubscribe(bidi_session, new_tab, wait_for_event, wait_for_future_safe, log_type):
    # Subscribe for log events globally
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_log(bidi_session, new_tab, log_type, "some text")
    await wait_for_future_safe(on_entry_added)

    # Unsubscribe from log events globally
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    expected_text_0 = await create_log(bidi_session, new_tab, log_type, "text_0")

    # Wait for some time before checking the events array
    await asyncio.sleep(0.5)
    assert len(events) == 0

    # Refresh to create a new context
    context = new_tab["context"]
    await bidi_session.browsing_context.navigate(context=context,
                                                 url='about:blank',
                                                 wait="complete")

    # Check we still don't receive ConsoleLogEntry events from the new context
    expected_text_1 = await create_log(bidi_session, new_tab, log_type, "text_1")

    # Wait for some time before checking the events array
    await asyncio.sleep(0.5)
    assert len(events) == 0

    # Refresh to create a new context. Note that we refresh to avoid getting
    # cached events from the log event buffer.
    context = new_tab["context"]
    await bidi_session.browsing_context.navigate(context=context,
                                                 url='about:blank',
                                                 wait="complete")

    # Check that if we subscribe again, we can receive events
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Check buffered events are emitted.
    assert len(events) == 2

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text_2 = await create_log(bidi_session, new_tab, log_type, "text_2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 3
    assert_base_entry(events[0], text=expected_text_0, context=new_tab["context"])
    assert_base_entry(events[1], text=expected_text_1, context=new_tab["context"])
    assert_base_entry(events[2], text=expected_text_2, context=new_tab["context"])

    # Check that we also get events from a new context
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text_3 = await create_log(bidi_session, new_context, log_type, "text_3")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 4
    assert_base_entry(events[3], text=expected_text_3, context=new_context["context"])

    remove_listener()
