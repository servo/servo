import asyncio
import pytest

from webdriver.error import TimeoutException
from webdriver.bidi.modules.script import ContextTarget

from tests.bidi import wait_for_bidi_events
from ... import int_interval
from .. import assert_navigation_info


pytestmark = pytest.mark.asyncio

NAVIGATION_COMMITTED_EVENT = "browsingContext.navigationCommitted"
PAGE_EMPTY = "/webdriver/tests/bidi/browsing_context/support/empty.html"
PAGE_REDIRECT_HTTP_EQUIV = (
    "/webdriver/tests/bidi/network/support/redirect_http_equiv.html"
)
PAGE_REDIRECTED_HTML = "/webdriver/tests/bidi/network/support/redirected.html"


async def test_unsubscribe(bidi_session):
    await bidi_session.session.subscribe(events=[NAVIGATION_COMMITTED_EVENT])
    await bidi_session.session.unsubscribe(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await bidi_session.browsing_context.create(type_hint="tab")

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_subscribe(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_COMMITTED_EVENT)
    url = inline("<div>foo</div>")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {
            "context": new_tab["context"],
            "navigation": result["navigation"],
            "url": url,
        },
    )


async def test_timestamp(
    bidi_session, current_time, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    time_start = await current_time()

    on_entry = wait_for_event(NAVIGATION_COMMITTED_EVENT)
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


async def test_basic_auth(
    bidi_session, subscribe_events, top_context, server_config, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_COMMITTED_EVENT)

    url_with_auth = "https://foo:bar@{0}:{1}{2}".format(
        server_config["browser_host"],
        server_config["ports"]["https"][0],
        PAGE_EMPTY
    )

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url_with_auth, wait="complete"
    )

    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {
            "context": top_context["context"],
            "navigation": result["navigation"],
            "url": url_with_auth
        },
    )


async def test_iframe(
    bidi_session, subscribe_events, top_context, test_page_same_origin_frame, test_page
):
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_same_origin_frame, wait="complete"
    )

    # Wait until we receive events for the top context and the iframe.
    await wait_for_bidi_events(bidi_session, events, 2)

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    children_info = root_info["children"]
    assert len(children_info) == 1

    # First navigation-committed event comes from the top-level browsing context.
    assert_navigation_info(
        events[0],
        {
            "context": top_context["context"],
            "navigation": result["navigation"],
            "url": test_page_same_origin_frame,
        },
    )

    assert_navigation_info(
        events[1],
        {
            "context": children_info[0]["context"],
            "url": test_page,
        },
    )
    assert events[1]["navigation"] is not None
    assert events[1]["navigation"] != result["navigation"]

    remove_listener()


async def test_nested_iframes(
    bidi_session,
    subscribe_events,
    top_context,
    test_page_nested_frames,
    test_page_same_origin_frame,
    test_page,
):
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    # Wait until we receive events for the top context and each of the 2 iframes.
    await wait_for_bidi_events(bidi_session, events, 3)

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])

    assert len(contexts) == 1
    root_info = contexts[0]
    assert len(root_info["children"]) == 1
    child1_info = root_info["children"][0]
    assert len(child1_info["children"]) == 1
    child2_info = child1_info["children"][0]

    assert_navigation_info(
        events[0],
        {
            "context": root_info["context"],
            "navigation": result["navigation"],
            "url": test_page_nested_frames,
        },
    )

    assert_navigation_info(
        events[1],
        {
            "context": child1_info["context"],
            "url": test_page_same_origin_frame,
        },
    )
    assert events[1]["navigation"] is not None
    assert events[1]["navigation"] != result["navigation"]

    assert_navigation_info(
        events[2],
        {
            "context": child2_info["context"],
            "url": test_page,
        },
    )
    assert events[2]["navigation"] is not None
    assert events[2]["navigation"] != result["navigation"]
    assert events[2]["navigation"] != events[1]["navigation"]

    remove_listener()


async def test_same_document(bidi_session, new_tab, url, subscribe_events):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(PAGE_EMPTY), wait="complete"
    )

    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(PAGE_EMPTY + "#foo"), wait="complete"
    )

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


@pytest.mark.parametrize("sandbox", [None, "sandbox_1"])
async def test_document_write(bidi_session, subscribe_events, new_tab, sandbox):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await bidi_session.script.evaluate(
        expression="""document.open(); document.write("<h1>Replaced</h1>"); document.close();""",
        target=ContextTarget(new_tab["context"], sandbox),
        await_promise=False,
    )

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_base_element(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_COMMITTED_EVENT)
    url = inline("""<base href="/relative-path">""")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {"context": new_tab["context"], "navigation": result["navigation"], "url": url},
    )


async def test_redirect_http_equiv(
    bidi_session, subscribe_events, top_context, url
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    # PAGE_REDIRECT_HTTP_EQUIV should redirect to PAGE_REDIRECTED_HTML immediately
    http_equiv_url = url(PAGE_REDIRECT_HTTP_EQUIV)
    redirected_url = url(PAGE_REDIRECTED_HTML)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=http_equiv_url,
        wait="complete",
    )

    # Wait until we receive two events, one for the initial navigation and one
    # for the http-equiv "redirect".
    await wait_for_bidi_events(bidi_session, events, 2)

    assert_navigation_info(
        events[0],
        {
            "context": top_context["context"],
            "url": http_equiv_url,
        },
    )
    assert_navigation_info(
        events[1],
        {
            "context": top_context["context"],
            "url": redirected_url,
        },
    )

    remove_listener()


async def test_redirect_navigation(
    bidi_session, subscribe_events, top_context, url
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    html_url = url(PAGE_EMPTY)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={html_url}"
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=redirect_url,
        wait="complete",
    )

    assert len(events) == 1
    assert_navigation_info(
        events[0],
        {
            "context": top_context["context"],
            "url": html_url,
        })

    remove_listener()


async def test_navigate_history_pushstate(
    bidi_session, inline, new_tab, subscribe_events, wait_for_event, wait_for_future_safe
):
    await subscribe_events([NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    url = inline("""
        <script>
            window.addEventListener('DOMContentLoaded', () => {
                history.pushState({}, '', '#1');
            });
        </script>""")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    with pytest.raises(TimeoutException):
        # Assert only a single event is emitted.
        await wait_for_bidi_events(bidi_session, events, 2, timeout=0.5)

    assert len(events) == 1
    assert events[0]["navigation"] == result["navigation"]

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, subscribe_events, type_hint):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await bidi_session.browsing_context.create(type_hint=type_hint)

    # In the future we can wait for "browsingContext.contextCreated" event instead.
    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_navigate_to_about_blank(
    bidi_session, subscribe_events, new_tab, wait_for_event, wait_for_future_safe
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    on_entry = wait_for_event(NAVIGATION_COMMITTED_EVENT)
    url = "about:blank"
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(
        event,
        {
            "context": new_tab["context"],
            "navigation": result["navigation"],
            "url": url,
        },
    )


@pytest.mark.parametrize("url", ["", "about:blank", "about:blank?test"])
async def test_window_open_with_about_blank(
    bidi_session, subscribe_events, top_context, url
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])

    # Track all received browsingContext.navigationCommitted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        NAVIGATION_COMMITTED_EVENT, on_event
    )

    await bidi_session.script.evaluate(
        expression=f"window.open('{url}');",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_window_open_with_url(
    bidi_session,
    subscribe_events,
    top_context,
    wait_for_event,
    inline,
    wait_for_future_safe,
):
    await subscribe_events(events=[NAVIGATION_COMMITTED_EVENT])
    on_navigation_committed = wait_for_event(NAVIGATION_COMMITTED_EVENT)
    url = inline("<div>foo</div>")

    await bidi_session.script.evaluate(
        expression=f"window.open('{url}');",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    event = await wait_for_future_safe(on_navigation_committed)

    result = await bidi_session.browsing_context.get_tree()

    assert_navigation_info(
        event,
        {
            "context": result[1]["context"],
            "url": url,
        },
    )
