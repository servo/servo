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
async def test_new_context(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    await subscribe_events([CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        children=None,
        url="about:blank",
        parent=None,
        user_context="default"
    )


async def test_evaluate_window_open_without_url(bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, top_context):
    await subscribe_events([CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression="""window.open();""",
        target=ContextTarget(top_context["context"]),
        await_promise=False)

    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        context=None,
        children=None,
        url="about:blank",
        parent=None,
        original_opener=top_context["context"],
    )


async def test_evaluate_window_open_with_url(bidi_session, subscribe_events, wait_for_event, wait_for_future_safe, inline, top_context):
    url = inline("<div>foo</div>")

    await subscribe_events([CONTEXT_CREATED_EVENT])

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)

    await bidi_session.script.evaluate(
        expression=f"""window.open("{url}");""",
        target=ContextTarget(top_context["context"]),
        await_promise=False)
    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        context=None,
        children=None,
        url="about:blank",
        parent=None,
        original_opener=top_context["context"],
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_event_emitted_before_create_returns(
    bidi_session, subscribe_events, type_hint
):
    # Subscribe before assigning the listener, as subscription emits the events
    # for already existing contexts.
    await subscribe_events([CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

    context = await bidi_session.browsing_context.create(type_hint=type_hint)

    # If the browsingContext.contextCreated event was emitted after the
    # browsingContext.create command resolved, the array would most likely be
    # empty at this point.
    assert len(events) == 1

    assert_browsing_context(
        events[0],
        context["context"],
        children=None,
        url="about:blank",
        parent=None,
        user_context="default",
    )

    remove_listener()


async def test_navigate_creates_iframes(bidi_session, subscribe_events, top_context, test_page_multiple_frames):
    # Subscribe before assigning the listener, as subscription emits the events
    # for already existing contexts.
    await subscribe_events([CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

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


async def test_navigate_creates_nested_iframes(bidi_session, subscribe_events, top_context, test_page_nested_frames):
    # Subscribe before assigning the listener, as subscription emits the events
    # for already existing contexts.
    await subscribe_events([CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

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


async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, top_context, test_page_same_origin_frame
):
    # Subscribe to a specific context
    await subscribe_events(
        events=[CONTEXT_CREATED_EVENT], contexts=[top_context["context"]]
    )

    # Track all received browsingContext.contextCreated events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

    await bidi_session.browsing_context.create(type_hint="tab")

    # Make sure we didn't receive the event for the new tab
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )

    # Make sure we received the event for the iframe
    await wait.until(lambda _: len(events) >= 1)
    assert len(events) == 1

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
    # Subscribe before assigning the listener, as subscription emits the events
    # for already existing contexts.
    await subscribe_events([CONTEXT_CREATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_CREATED_EVENT, on_event)

    user_context = await create_user_context()
    assert len(events) == 0

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    context = await bidi_session.browsing_context.create(
        type_hint=type_hint, user_context=user_context
    )
    context_info = await wait_for_future_safe(on_entry)

    assert len(events) == 1

    assert_browsing_context(
        context_info,
        context["context"],
        children=None,
        url="about:blank",
        parent=None,
        user_context=user_context,
    )

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_existing_context(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    # See https://w3c.github.io/webdriver-bidi/#ref-for-remote-end-subscribe-steps%E2%91%A1.
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    await subscribe_events([CONTEXT_CREATED_EVENT], contexts=[top_level_context["context"]])
    context_info = await wait_for_future_safe(on_entry)

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        children=None,
        url="about:blank",
        parent=None,
        user_context="default"
    )
