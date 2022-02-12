import pytest

import time

from . import assert_base_entry, create_log


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_subscribe_twice(bidi_session,
                               current_session,
                               inline,
                               wait_for_event,
                               log_type):
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
    expected_text = create_log(current_session, inline, log_type, "text1")
    await on_entry_added

    assert len(events) == 1
    assert_base_entry(events[0], text=expected_text)

    # Wait for some time and check the events array again
    time.sleep(0.5)
    assert len(events) == 1;

    remove_listener()


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_subscribe_unsubscribe(bidi_session,
                                     current_session,
                                     inline,
                                     wait_for_event,
                                     log_type):
    # Subscribe for log events globally
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    create_log(current_session, inline, log_type, "text1")
    await on_entry_added

    # Unsubscribe from log events globally
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Track all received log.entryAdded events in the events array
    events = []
    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    create_log(current_session, inline, log_type, "text2")

    # Wait for some time before checking the events array
    time.sleep(0.5)
    assert len(events) == 0;

    # Refresh to create a new context
    current_session.refresh()

    # Check we still don't receive ConsoleLogEntry events from the new context
    create_log(current_session, inline, log_type, "text3")

    # Wait for some time before checking the events array
    time.sleep(0.5)
    assert len(events) == 0;

    # Refresh to create a new context. Note that we refresh to avoid getting
    # cached events from the log event buffer.
    current_session.refresh()

    # Check that if we subscribe again, we can receive events
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = create_log(current_session, inline, log_type, "text4")
    await on_entry_added

    assert len(events) == 1;
    assert_base_entry(events[0], text=expected_text)

    # Check that we also get events from new tab/window
    current_session.new_window()

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = create_log(current_session, inline, log_type, "text5")
    await on_entry_added

    assert len(events) == 2;
    assert_base_entry(events[1], text=expected_text)

    remove_listener()
