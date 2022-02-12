import pytest
import time

from . import assert_base_entry, create_log

@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_console_log_cached_messages(bidi_session,
                                           current_session,
                                           wait_for_event,
                                           inline,
                                           log_type):
    # Unsubscribe in case previous tests subscribed to log.entryAdded
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Refresh to make sure no events are cached for the current window global
    # from previous tests.
    current_session.refresh()

    # Log a message before subscribing
    expected_text = create_log(current_session, inline, log_type, "cached_message")

    # Track all received log.entryAdded events in the events array
    events = []
    async def on_event(method, data):
        events.append(data)
    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Subscribe
    on_entry_added = wait_for_event("log.entryAdded")
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await on_entry_added
    assert len(events) == 1;

    # Check the log.entryAdded event received has the expected properties.
    assert_base_entry(events[0], text=expected_text)

    # Unsubscribe and re-subscribe
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Wait for some time to catch all messages.
    time.sleep(0.5)

    # Check that the cached event was not re-emitted.
    assert len(events) == 1;

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = create_log(current_session, inline, log_type, "live_message")
    await on_entry_added

    # Check that we only received the live message.
    assert len(events) == 2;
    assert_base_entry(events[1], text=expected_text)

    # Unsubscribe, log a message and re-subscribe
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])
    expected_text = create_log(current_session, inline, log_type, "cached_message_2")

    on_entry_added = wait_for_event("log.entryAdded")
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await on_entry_added

    # Check that only the newly cached event was emitted
    assert len(events) == 3;
    assert_base_entry(events[2], text=expected_text)

    remove_listener()


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_console_log_cached_message_after_refresh(bidi_session,
                                                        current_session,
                                                        wait_for_event,
                                                        inline,
                                                        log_type):
    # Unsubscribe in case previous tests subscribed to log.entryAdded
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Refresh to make sure no events are cached for the current window global
    # from previous tests.
    current_session.refresh()

    # Track all received log.entryAdded events in the events array
    events = []
    async def on_event(method, data):
        events.append(data)
    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Log a message, refresh, log another message and subscribe
    create_log(current_session, inline, log_type, "missed_message")
    current_session.refresh();
    expected_text = create_log(current_session, inline, log_type, "cached_message")

    on_entry_added = wait_for_event("log.entryAdded")
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await on_entry_added

    # Wait for some time to catch all messages.
    time.sleep(0.5)

    # Check that only the cached message was retrieved.
    assert len(events) == 1;
    assert_base_entry(events[0], text=expected_text)

    remove_listener()

