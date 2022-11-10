import asyncio

import pytest
from webdriver.error import TimeoutException
from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll
from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"


async def test_not_unsubscribed(bidi_session):
    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    # Track all received browsingContext.contextCreated events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.create(type_hint="tab")

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, wait_for_event, type_hint):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    top_level_context = await bidi_session.browsing_context.create(type_hint="tab")
    context_info = await on_entry

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        children=None,
        url="about:blank",
        parent=None,
    )


async def test_evaluate_window_open_without_url(bidi_session, wait_for_event, top_context):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression="""window.open();""",
        target=ContextTarget(top_context["context"]),
        await_promise=False)

    context_info = await on_entry

    assert_browsing_context(
        context_info,
        context=None,
        children=None,
        url="about:blank",
        parent=None,
    )

    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])


async def test_evaluate_window_open_with_url(bidi_session, wait_for_event, inline, top_context):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    url = inline("<div>foo</div>")

    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression=f"""window.open("{url}");""",
        target=ContextTarget(top_context["context"]),
        await_promise=False)
    context_info = await on_entry

    assert_browsing_context(
        context_info,
        context=None,
        children=None,
        url="about:blank",
        parent=None,
    )


async def test_navigate_creates_iframes(bidi_session, top_context, test_page_multiple_frames):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)
    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    wait = AsyncPoll(
        bidi_session, message="Didn't receive context created events for frames"
    )
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    # Get all browsing contexts from the first tab
    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    children_info = root_info["children"]
    assert len(children_info) == 2

    # Note: Live `browsingContext.contextCreated` events are always created with "about:blank":
    # https://github.com/w3c/webdriver-bidi/issues/220#issuecomment-1145785349
    assert_browsing_context(
        events[0],
        children_info[0]["context"],
        children=None,
        url="about:blank",
        parent=root_info["context"],
    )

    assert_browsing_context(
        events[1],
        children_info[1]["context"],
        children=None,
        url="about:blank",
        parent=root_info["context"],
    )

    remove_listener()
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])


async def test_navigate_creates_nested_iframes(bidi_session, top_context, test_page_nested_frames):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)
    await bidi_session.session.subscribe(events=[CONTEXT_CREATED_EVENT])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    wait = AsyncPoll(
        bidi_session, message="Didn't receive context created events for frames"
    )
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    # Get all browsing contexts from the first tab
    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    assert len(root_info["children"]) == 1
    child1_info = root_info["children"][0]
    assert len(child1_info["children"]) == 1
    child2_info = child1_info["children"][0]

    # Note: `browsingContext.contextCreated` is always created with "about:blank":
    # https://github.com/w3c/webdriver-bidi/issues/220#issuecomment-1145785349
    assert_browsing_context(
        events[0],
        child1_info["context"],
        children=None,
        url="about:blank",
        parent=root_info["context"],
    )

    assert_browsing_context(
        events[1],
        child2_info["context"],
        children=None,
        url="about:blank",
        parent=child1_info["context"],
    )

    remove_listener()
    await bidi_session.session.unsubscribe(events=[CONTEXT_CREATED_EVENT])
