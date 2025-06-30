import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException

from tests.bidi import wait_for_bidi_events
from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio

CONTEXT_DESTROYED_EVENT = "browsingContext.contextDestroyed"


async def test_unsubscribe(bidi_session, new_tab):
    await bidi_session.session.subscribe(events=[CONTEXT_DESTROYED_EVENT])
    await bidi_session.session.unsubscribe(events=[CONTEXT_DESTROYED_EVENT])

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.close(context=new_tab["context"])

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    contexts = await bidi_session.browsing_context.get_tree(root=new_context["context"])

    await bidi_session.browsing_context.close(context=new_context["context"])

    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        new_context["context"],
        children=0,
        url="about:blank",
        parent=None,
        user_context="default",
        client_window=contexts[0]["clientWindow"],
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_navigate(bidi_session, subscribe_events, new_tab, inline, domain):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    url = inline("<div>test</div>", domain=domain)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    # Make sure navigation doesn't cause the context to be destroyed
    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_navigate_iframe(
    bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, new_tab, inline, domain
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)

    frame_url = inline("<div>foo</div>")
    url = inline(f"<iframe src='{frame_url}'></iframe>")
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    frame = contexts[0]["children"][0]

    # Navigate to destroy iframes
    url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        frame["context"],
        children=0,
        url=frame_url,
        parent=new_tab["context"],
        client_window=contexts[0]["clientWindow"],
    )


async def test_delete_iframe(
    bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, new_tab, inline, test_page_multiple_frames
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)

    await bidi_session.browsing_context.navigate(
        url=test_page_multiple_frames, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe = contexts[0]["children"][0]

    # Delete the first iframe
    await bidi_session.script.evaluate(
        expression="""document.querySelector('iframe:nth-of-type(1)').remove()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        iframe["context"],
        children=0,
        url=iframe["url"],
        parent=new_tab["context"],
        client_window=contexts[0]["clientWindow"]
    )


async def test_nested_iframes_delete_top_iframe(
    bidi_session,
    subscribe_events,
    new_tab,
    test_page_nested_frames,
    test_page_same_origin_frame,
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])
    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        url=test_page_nested_frames, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    top_iframe = contexts[0]["children"][0]

    # Delete top iframe
    await bidi_session.script.evaluate(
        expression="""document.querySelector('iframe').remove()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    assert len(events) == 1
    assert_browsing_context(
        events[0],
        top_iframe["context"],
        children=1,
        url=test_page_same_origin_frame,
        parent=new_tab["context"],
        client_window=contexts[0]["clientWindow"]
    )

    remove_listener()


async def test_nested_iframes_delete_deepest_iframe(
    bidi_session,
    subscribe_events,
    new_tab,
    test_page_nested_frames,
    test_page_same_origin_frame,
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])
    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        url=test_page_nested_frames, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    top_iframe = contexts[0]["children"][0]
    deepest_iframe = contexts[0]["children"][0]["children"][0]

    # Delete deepest iframe
    await bidi_session.script.evaluate(
        expression="""document.querySelector('iframe').remove()""",
        target=ContextTarget(top_iframe["context"]),
        await_promise=False,
    )

    assert len(events) == 1
    assert_browsing_context(
        events[0],
        deepest_iframe["context"],
        children=0,
        url=deepest_iframe["url"],
        parent=top_iframe["context"],
        client_window=contexts[0]["clientWindow"],
    )

    remove_listener()


async def test_iframe_destroy_parent(
    bidi_session, subscribe_events, new_tab, test_page_nested_frames
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])
    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(
        url=test_page_nested_frames, context=new_tab["context"], wait="complete"
    )
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    # Destroy top context
    await bidi_session.browsing_context.close(context=new_tab["context"])

    assert len(events) == 1
    assert_browsing_context(
        events[0],
        new_tab["context"],
        children=1,
        url=test_page_nested_frames,
        parent=None,
        client_window=contexts[0]["clientWindow"],
    )

    remove_listener()


async def test_subscribe_to_one_context(bidi_session, subscribe_events, new_tab):
    # Subscribe to a specific context
    await subscribe_events(
        events=[CONTEXT_DESTROYED_EVENT], contexts=[new_tab["context"]]
    )

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    another_new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.close(context=another_new_tab["context"])

    # Make sure we didn't receive the event for the new tab
    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    await bidi_session.browsing_context.close(context=new_tab["context"])

    # Make sure we received the event
    await wait_for_bidi_events(bidi_session, events, 1)

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_user_context(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    subscribe_events,
    create_user_context,
    type_hint,
):
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    user_context = await create_user_context()
    assert len(events) == 0

    context = await bidi_session.browsing_context.create(
        type_hint=type_hint, user_context=user_context
    )
    contexts = await bidi_session.browsing_context.get_tree(root=context["context"])
    assert len(events) == 0

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)
    await bidi_session.browsing_context.close(context=context["context"])
    context_info = await wait_for_future_safe(on_entry)
    assert len(events) == 1

    assert_browsing_context(
        context_info,
        context["context"],
        children=0,
        url="about:blank",
        parent=None,
        user_context=user_context,
        client_window=contexts[0]["clientWindow"],
    )

    remove_listener()


async def test_with_user_context_subscription(
    bidi_session,
    subscribe_events,
    create_user_context,
    wait_for_events
):
    user_context = await create_user_context()

    await subscribe_events(
        events=[CONTEXT_DESTROYED_EVENT], user_contexts=[user_context]
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )
    contexts = await bidi_session.browsing_context.get_tree(root=context["context"])

    with wait_for_events([CONTEXT_DESTROYED_EVENT]) as waiter:
        await bidi_session.browsing_context.close(context=context["context"])
        events = await waiter.get_events(lambda events: len(events) >= 1)
        assert len(events) == 1

        assert_browsing_context(
            events[0][1],
            context["context"],
            children=0,
            url="about:blank",
            parent=None,
            user_context=user_context,
            client_window=contexts[0]["clientWindow"]
        )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_client_window(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    contexts = await bidi_session.browsing_context.get_tree(root=top_level_context["context"])

    await bidi_session.browsing_context.close(context=top_level_context["context"])
    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        children=0,
        url="about:blank",
        parent=None,
        user_context="default",
        client_window=contexts[0]["clientWindow"]
    )
