import pytest

from . import assert_base_entry, create_log


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_console_log_cached_messages(
    bidi_session, wait_for_event, wait_for_future_safe, log_type, new_tab
):
    # Clear events buffer.
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Log a message before subscribing
    expected_text = await create_log(bidi_session, new_tab, log_type, "cached_message")

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Subscribe
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    # Cached events are emitted before the subscribe command is finished.
    assert len(events) == 1

    # Check the log.entryAdded event received has the expected properties.
    assert_base_entry(events[0], text=expected_text, context=new_tab["context"])

    # Unsubscribe and re-subscribe
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Check that the cached event was not re-emitted.
    assert len(events) == 1

    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_log(bidi_session, new_tab, log_type, "live_message")
    await wait_for_future_safe(on_entry_added)

    # Check that we only received the live message.
    assert len(events) == 2
    assert_base_entry(events[1], text=expected_text, context=new_tab["context"])

    # Unsubscribe, log a message and re-subscribe
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])
    expected_text = await create_log(bidi_session, new_tab, log_type, "cached_message_2")

    await bidi_session.session.subscribe(events=["log.entryAdded"])

    # Check that only the newly cached event was emitted
    assert len(events) == 3
    assert_base_entry(events[2], text=expected_text, context=new_tab["context"])

    await bidi_session.session.unsubscribe(events=["log.entryAdded"])
    remove_listener()


@pytest.mark.asyncio
@pytest.mark.parametrize("log_type", ["console_api_log", "javascript_error"])
async def test_console_log_cached_message_after_refresh(
    bidi_session, subscribe_events, new_tab, log_type
):
    # Clear events buffer.
    await bidi_session.session.subscribe(events=["log.entryAdded"])
    await bidi_session.session.unsubscribe(events=["log.entryAdded"])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Log a message, refresh, log another message and subscribe
    expected_text_1 = await create_log(bidi_session, new_tab, log_type, "cached_message_1")
    context = new_tab["context"]
    await bidi_session.browsing_context.navigate(context=context,
                                                 url='about:blank',
                                                 wait="complete")
    expected_text_2 = await create_log(bidi_session, new_tab, log_type, "cached_message_2")

    await subscribe_events(events=["log.entryAdded"])

    # Check that only the cached message was retrieved.
    assert len(events) == 2
    assert_base_entry(events[0], text=expected_text_1)
    assert_base_entry(events[1], text=expected_text_2)

    remove_listener()
