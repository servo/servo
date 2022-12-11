import pytest

from tests.support.sync import AsyncPoll
from .. import assert_navigation_info

pytestmark = pytest.mark.asyncio

DOM_CONTENT_LOADED_EVENT = "browsingContext.domContentLoaded"


async def test_unsubscribe(bidi_session, inline, top_context):
    await bidi_session.session.subscribe(events=[DOM_CONTENT_LOADED_EVENT])
    await bidi_session.session.unsubscribe(events=[DOM_CONTENT_LOADED_EVENT])

    # Track all received browsingContext.domContentLoaded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        DOM_CONTENT_LOADED_EVENT, on_event
    )

    url = inline("<div>foo</div>")

    # When navigation reaches complete state,
    # we should have received a browsingContext.domContentLoaded event
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    assert len(events) == 0

    remove_listener()


async def test_subscribe(bidi_session, subscribe_events, inline, new_tab, wait_for_event):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=url)
    event = await on_entry

    assert_navigation_info(event, new_tab["context"], url)


async def test_iframe(bidi_session, subscribe_events, new_tab, test_page, test_page_same_origin_frame):
    events = []

    async def on_event(method, data):
        # Filter out events for about:blank to avoid browser differences
        if data["url"] != 'about:blank':
            events.append(data)

    remove_listener = bidi_session.add_event_listener(
        DOM_CONTENT_LOADED_EVENT, on_event
    )
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=test_page_same_origin_frame
    )

    wait = AsyncPoll(
        bidi_session, message="Didn't receive dom content loaded events for frames"
    )
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    assert len(root_info["children"]) == 1
    child_info = root_info["children"][0]

    # The ordering of the domContentLoaded event is not guaranteed between the
    # root page and the iframe, find the appropriate events in the current list.
    first_is_root = events[0]["context"] == root_info["context"]
    root_event = events[0] if first_is_root else events[1]
    child_event = events[1] if first_is_root else events[0]

    assert_navigation_info(root_event, root_info["context"], test_page_same_origin_frame)
    assert_navigation_info(child_event, child_info["context"], test_page)

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, subscribe_events, wait_for_event, type_hint):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    event = await on_entry

    assert_navigation_info(event, new_context["context"], "about:blank")
