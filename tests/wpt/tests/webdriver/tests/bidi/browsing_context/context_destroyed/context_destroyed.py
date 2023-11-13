import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll
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

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, wait_for_event, subscribe_events, type_hint):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.close(context=new_context["context"])

    context_info = await on_entry

    assert_browsing_context(
        context_info,
        new_context["context"],
        children=None,
        url="about:blank",
        parent=None,
    )


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_navigate(bidi_session, subscribe_events, new_tab, inline, domain):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_DESTROYED_EVENT, on_event)

    url = inline(f"<div>test</div>", domain=domain)
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    # Make sure navigation doesn't cause the context to be destroyed
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_navigate_iframe(
    bidi_session, wait_for_event, subscribe_events, new_tab, inline, domain
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

    context_info = await on_entry

    assert_browsing_context(
        context_info,
        frame["context"],
        children=None,
        url=frame_url,
        parent=new_tab["context"],
    )


async def test_delete_iframe(
    bidi_session, wait_for_event, subscribe_events, new_tab, inline
):
    await subscribe_events([CONTEXT_DESTROYED_EVENT])

    on_entry = wait_for_event(CONTEXT_DESTROYED_EVENT)

    frame_url = inline("<div>foo</div>")
    url = inline(f"<iframe src='{frame_url}'></iframe>")
    await bidi_session.browsing_context.navigate(
        url=url, context=new_tab["context"], wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe = contexts[0]["children"][0]

    # Delete the iframe
    await bidi_session.script.evaluate(
        expression="""document.querySelector('iframe').remove()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    context_info = await on_entry

    assert_browsing_context(
        context_info,
        iframe["context"],
        children=None,
        url=frame_url,
        parent=new_tab["context"],
    )


async def test_delete_nested_iframes(
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
        children=None,
        url=test_page_same_origin_frame,
        parent=new_tab["context"],
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

    # Destroy top context
    await bidi_session.browsing_context.close(context=new_tab["context"])

    assert len(events) == 1
    assert_browsing_context(
        events[0],
        new_tab["context"],
        children=None,
        url=test_page_nested_frames,
        parent=None,
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
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    await bidi_session.browsing_context.close(context=new_tab["context"])

    # Make sure we received the event
    await wait.until(lambda _: len(events) >= 1)
    assert len(events) == 1

    remove_listener()
