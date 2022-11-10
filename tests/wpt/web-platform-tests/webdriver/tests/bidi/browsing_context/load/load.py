import pytest
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll
from .. import assert_navigation_info

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


async def test_not_unsubscribed(bidi_session, inline, top_context):
    await bidi_session.session.subscribe(events=[CONTEXT_LOAD_EVENT])
    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])

    # Track all received browsingContext.load events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_LOAD_EVENT, on_event)

    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url
    )

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


async def test_subscribe(bidi_session, inline, new_tab, wait_for_event):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])

    await bidi_session.session.subscribe(events=[CONTEXT_LOAD_EVENT])

    on_entry = wait_for_event(CONTEXT_LOAD_EVENT)
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=url)
    event = await on_entry

    assert_navigation_info(event, new_tab["context"], url)

    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])


async def test_iframe(bidi_session, new_tab, test_page, test_page_same_origin_frame):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(CONTEXT_LOAD_EVENT, on_event)
    await bidi_session.session.subscribe(events=[CONTEXT_LOAD_EVENT])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page_same_origin_frame
    )

    wait = AsyncPoll(
        bidi_session, message="Didn't receive context load events for frames"
    )
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    assert len(root_info["children"]) == 1
    child_info = root_info["children"][0]

    # First load event comes from iframe
    assert_navigation_info(events[0], child_info["context"], test_page)
    assert_navigation_info(events[1], root_info["context"], test_page_same_origin_frame)

    remove_listener()
    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, wait_for_event, type_hint):
    # Unsubscribe in case a previous tests subscribed to the event
    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])

    await bidi_session.session.subscribe(events=[CONTEXT_LOAD_EVENT])

    on_entry = wait_for_event(CONTEXT_LOAD_EVENT)
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    event = await on_entry

    assert_navigation_info(event, new_context["context"], "about:blank")

    await bidi_session.session.unsubscribe(events=[CONTEXT_LOAD_EVENT])
