import pytest
from webdriver.bidi.modules.script import ContextTarget

from ... import create_console_api_message, recursive_compare

pytestmark = pytest.mark.asyncio


# The basic use case of subscribing to all contexts for a single event
# is covered by tests for each event in the dedicated folders.


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
    on_entry_added = wait_for_event("log.entryAdded")
    expected_text = await create_console_api_message(bidi_session, new_tab, "text3")
    await wait_for_future_safe(on_entry_added)

    assert len(events) == 2
    recursive_compare(
        {
            "text": expected_text,
        },
        events[1],
    )

    on_entry_added = wait_for_event("log.entryAdded")
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


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_subscribe_to_child_context_and_navigate_sibling_frame(
    bidi_session,
    subscribe_events,
    wait_for_event, wait_for_future_safe,
    top_context,
    test_page_multiple_frames,
    inline,
    domain,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 2
    frame_1 = contexts[0]["children"][0]
    frame_2 = contexts[0]["children"][1]

    # Subscribe to the first frame context
    await subscribe_events(events=["browsingContext.load"], contexts=[frame_1["context"]])

    # Track all received browsingContext.load events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("browsingContext.load", on_event)

    # Navigate the second iframe
    url = inline("<div>foo</div>", domain=domain)
    on_load = wait_for_event("browsingContext.load")
    await bidi_session.script.evaluate(
        expression=f"document.querySelectorAll('iframe')[1].src = '{url}'",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    await wait_for_future_safe(on_load)

    # Make sure we received the event for the second iframe
    assert len(events) == 1
    assert events[0]["context"] == frame_2["context"]
    assert events[0]["url"] == url

    remove_listener()


async def test_subscribe_to_child_context_and_navigate_top_context(
    bidi_session,
    subscribe_events,
    wait_for_event, wait_for_future_safe,
    top_context,
    test_page_multiple_frames,
    test_page,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 2
    frame_1 = contexts[0]["children"][0]

    # Subscribe to the first frame context
    await subscribe_events(events=["browsingContext.load"], contexts=[frame_1["context"]])

    # Track all received browsingContext.load events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("browsingContext.load", on_event)

    # Navigate the top context, which removes the first iframe
    on_load = wait_for_event("browsingContext.load")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page, wait="complete"
    )
    await wait_for_future_safe(on_load)

    # Make sure we received the event for the top context
    assert len(events) == 1
    assert events[0]["context"] == top_context["context"]
    assert events[0]["url"] == test_page

    remove_listener()


async def test_subscribe_to_child_context_and_navigate_nested_frame(
    bidi_session,
    subscribe_events,
    wait_for_event, wait_for_future_safe,
    top_context,
    test_page_nested_frames,
    inline,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts[0]["children"]) == 1
    frame_level_1 = contexts[0]["children"][0]

    assert len(frame_level_1["children"]) == 1
    frame_level_2 = frame_level_1["children"][0]

    # Subscribe to the first level frame context
    await subscribe_events(events=["browsingContext.load"], contexts=[frame_level_1["context"]])

    # Track all received browsingContext.load events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("browsingContext.load", on_event)

    # Navigate the second level frame
    url = inline("<div>foo</div>")
    on_load = wait_for_event("browsingContext.load")
    await bidi_session.script.evaluate(
        expression=f"document.querySelector('iframe').contentDocument.querySelector('iframe').src = '{url}'",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    await wait_for_future_safe(on_load)

    # Make sure we received the event for the second level frame
    assert len(events) == 1
    assert events[0]["context"] == frame_level_2["context"]
    assert events[0]["url"] == url

    remove_listener()
