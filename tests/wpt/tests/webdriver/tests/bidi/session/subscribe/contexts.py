import pytest

from ... import create_console_api_message, recursive_compare


# The basic use case of subscribing to all contexts for a single event
# is covered by tests for each event in the dedicated folders.


@pytest.mark.asyncio
async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, top_context, new_tab, wait_for_event, wait_for_future_safe
):
    # Subscribe for log events to a specific context
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the another context
    await create_console_api_message(bidi_session, new_tab, "text1")

    assert len(events) == 0

    # Trigger another console event in the observed context
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, top_context, "text2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    recursive_compare(
        {
            "text": expected_text,
        },
        events[0],
    )

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_one_context_twice(
    bidi_session, subscribe_events, top_context, wait_for_event, wait_for_future_safe
):
    # Subscribe twice for log events to a specific context
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger a console event in the observed context
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, top_context, "text2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    recursive_compare(
        {
            "text": expected_text,
        },
        events[0],
    )

    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_one_context_and_then_to_all(
    bidi_session, subscribe_events, top_context, new_tab, wait_for_event, wait_for_future_safe
):
    # Subscribe for log events to a specific context
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the another context
    buffered_event_expected_text = await create_console_api_message(
        bidi_session, new_tab, "text1"
    )

    assert len(events) == 0

    # Trigger another console event in the observed context
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, top_context, "text2")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 1
    recursive_compare(
        {
            "text": expected_text,
        },
        events[0],
    )

    events = []

    # Subscribe to all contexts
    await subscribe_events(events=["log.entryAdded"])

    # Check that we received the buffered event
    assert len(events) == 1
    recursive_compare(
        {
            "text": buffered_event_expected_text,
        },
        events[0],
    )

    # Trigger again events in each context
    expected_text = await create_console_api_message(bidi_session, new_tab, "text3")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 2
    recursive_compare(
        {
            "text": expected_text,
        },
        events[1],
    )

    expected_text = await create_console_api_message(bidi_session, top_context, "text4")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 3
    recursive_compare(
        {
            "text": expected_text,
        },
        events[2],
    )

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_all_context_and_then_to_one_again(
    bidi_session, subscribe_events, top_context, new_tab, wait_for_event, wait_for_future_safe
):
    # Subscribe to all contexts
    await subscribe_events(events=["log.entryAdded"])
    # Subscribe to one of the contexts again
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the context to which we tried to subscribe twice
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, top_context, "text1")
    await wait_for_future_safe(on_entry_added)

    # Make sure we received only one event
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_top_context_with_iframes(
    bidi_session,
    subscribe_events,
    wait_for_event, wait_for_future_safe,
    top_context,
    test_page_multiple_frames,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 2
    frame_1 = contexts[0]["children"][0]
    frame_2 = contexts[0]["children"][1]

    # Subscribe to the top context
    await subscribe_events(events=["log.entryAdded"], contexts=[top_context["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the first iframe
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, frame_1, "text1")
    await wait_for_future_safe(on_entry_added)

    # Make sure we received the event
    assert len(events) == 1

    # Trigger console event in the second iframe
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, frame_2, "text2")
    await wait_for_future_safe(on_entry_added)

    # Make sure we received the second event as well
    assert len(events) == 2

    remove_listener()


@pytest.mark.asyncio
async def test_subscribe_to_child_context(
    bidi_session,
    subscribe_events,
    wait_for_event, wait_for_future_safe,
    top_context,
    test_page_multiple_frames,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 2
    frame_1 = contexts[0]["children"][0]
    frame_2 = contexts[0]["children"][1]

    # Subscribe to the first frame context
    await subscribe_events(events=["log.entryAdded"], contexts=[frame_1["context"]])

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger console event in the top context
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, top_context, "text1")
    await wait_for_future_safe(on_entry_added)

    # Make sure we received the event
    assert len(events) == 1

    # Trigger console event in the second iframe
    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message(bidi_session, frame_2, "text2")
    await wait_for_future_safe(on_entry_added)

    # Make sure we received the second event as well
    assert len(events) == 2

    remove_listener()
