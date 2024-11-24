import pytest
from tests.support.sync import AsyncPoll
from webdriver.error import TimeoutException
from webdriver.bidi.modules.script import ContextTarget

from ... import int_interval
from .. import assert_navigation_info

pytestmark = pytest.mark.asyncio

DOM_CONTENT_LOADED_EVENT = "browsingContext.domContentLoaded"


async def test_unsubscribe(bidi_session, inline, top_context):
    # test
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


async def test_subscribe(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)
    url = inline("<div>foo</div>")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {
            "context": new_tab["context"],
            "url": url,
            "navigation": result["navigation"],
        },
    )


async def test_timestamp(
    bidi_session, current_time, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    time_start = await current_time()

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)
    url = inline("<div>foo</div>")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    time_end = await current_time()

    assert_navigation_info(
        event,
        {
            "context": new_tab["context"],
            "navigation": result["navigation"],
            "timestamp": int_interval(time_start, time_end),
        },
    )


async def test_iframe(
    bidi_session, subscribe_events, new_tab, test_page, test_page_same_origin_frame
):
    events = []

    async def on_event(method, data):
        # Filter out events for about:blank to avoid browser differences
        if data["url"] != "about:blank":
            events.append(data)

    remove_listener = bidi_session.add_event_listener(
        DOM_CONTENT_LOADED_EVENT, on_event
    )
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    result = await bidi_session.browsing_context.navigate(
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

    assert_navigation_info(
        root_event,
        {
            "context": root_info["context"],
            "url": test_page_same_origin_frame,
            "navigation": result["navigation"],
        },
    )
    assert_navigation_info(
        child_event, {"context": child_info["context"], "url": test_page}
    )
    assert child_event["navigation"] is not None
    assert child_event["navigation"] != root_event["navigation"]

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context_not_emitted(bidi_session, subscribe_events,
      wait_for_event, wait_for_future_safe, type_hint):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    # Track all received browsingContext.domContentLoaded events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        DOM_CONTENT_LOADED_EVENT, on_event
    )

    await bidi_session.browsing_context.create(type_hint=type_hint)

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("sandbox", [None, "sandbox_1"])
async def test_document_write(
      bidi_session, subscribe_events, new_tab, wait_for_event, wait_for_future_safe, sandbox
):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)

    await bidi_session.script.evaluate(
        expression="""document.open(); document.write("<h1>Replaced</h1>"); document.close();""",
        target=ContextTarget(new_tab["context"], sandbox),
        await_promise=False,
    )

    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {"context": new_tab["context"]},
    )
    assert event["navigation"] is not None


async def test_early_same_document_navigation(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)

    url = inline(
        """
        <script type="text/javascript">
            history.replaceState(null, 'initial', window.location.href);
        </script>
    """
    )

    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )

    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {"context": new_tab["context"], "navigation": result["navigation"], "url": url},
    )


async def test_page_with_base_tag(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[DOM_CONTENT_LOADED_EVENT])

    on_entry = wait_for_event(DOM_CONTENT_LOADED_EVENT)
    url = inline("""<base href="/relative-path">""")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {"context": new_tab["context"], "navigation": result["navigation"], "url": url},
    )
