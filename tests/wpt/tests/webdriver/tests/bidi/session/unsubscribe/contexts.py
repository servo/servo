import pytest

from ... import create_console_api_message, recursive_compare


# The basic use case of unsubscribing from all contexts for a single event
# is covered by tests for each event in the dedicated folders.


@pytest.mark.asyncio
async def test_unsubscribe_from_one_context(
    bidi_session, top_context, new_tab, wait_for_event, wait_for_future_safe
):
    # Subscribe for log events to multiple contexts
    await bidi_session.session.subscribe(
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

    remove_listener()
    await bidi_session.session.unsubscribe(
        events=["log.entryAdded"], contexts=[new_tab["context"]]
    )


@pytest.mark.asyncio
async def test_unsubscribe_from_top_context_with_iframes(
    bidi_session,
    top_context,
    test_page_same_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]

    # Subscribe and unsubscribe to the top context
    await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )
    await bidi_session.session.unsubscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger the event in the frame
    await create_console_api_message(bidi_session, frame, "text1")

    assert len(events) == 0

    remove_listener()


@pytest.mark.asyncio
async def test_unsubscribe_from_child_context(
    bidi_session,
    top_context,
    test_page_same_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]

    # Subscribe to top context
    await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )
    # Unsubscribe from the frame context
    await bidi_session.session.unsubscribe(
        events=["log.entryAdded"], contexts=[frame["context"]]
    )

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger the event in the frame
    await create_console_api_message(bidi_session, frame, "text1")
    # Trigger the event in the top context
    await create_console_api_message(bidi_session, top_context, "text2")

    # Make sure we didn't receive any of the triggered events
    assert len(events) == 0

    remove_listener()


@pytest.mark.asyncio
async def test_unsubscribe_from_one_context_after_navigation(
    bidi_session, top_context, test_alt_origin
):
    await bidi_session.session.subscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_alt_origin, wait="complete"
    )

    await bidi_session.session.unsubscribe(
        events=["log.entryAdded"], contexts=[top_context["context"]]
    )

    # Track all received log.entryAdded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("log.entryAdded", on_event)

    # Trigger the event
    await create_console_api_message(bidi_session, top_context, "text1")

    # Make sure we successfully unsubscribed
    assert len(events) == 0

    remove_listener()
