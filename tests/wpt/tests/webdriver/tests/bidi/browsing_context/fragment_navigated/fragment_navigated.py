import pytest

from tests.support.sync import AsyncPoll
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException

from ... import any_int, recursive_compare, int_interval
from .. import assert_navigation_info

pytestmark = pytest.mark.asyncio

EMPTY_PAGE = "/webdriver/tests/bidi/browsing_context/support/empty.html"
FRAGMENT_NAVIGATED_EVENT = "browsingContext.fragmentNavigated"


async def test_unsubscribe(bidi_session, url, top_context):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url(EMPTY_PAGE), wait="complete"
    )

    await bidi_session.session.subscribe(events=[FRAGMENT_NAVIGATED_EVENT])
    await bidi_session.session.unsubscribe(events=[FRAGMENT_NAVIGATED_EVENT])

    # Track all received browsingContext.fragmentNavigated events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )

    # When navigation reaches complete state,
    # we should have received a browsingContext.fragmentNavigated event
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url(EMPTY_PAGE + '#foo'), wait="complete"
    )

    assert len(events) == 0

    remove_listener()


async def test_subscribe(bidi_session, subscribe_events, url, new_tab, wait_for_event, wait_for_future_safe):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE), wait="complete"
    )

    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    on_entry = wait_for_event(FRAGMENT_NAVIGATED_EVENT)
    target_url = url(EMPTY_PAGE + '#foo')
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=target_url, wait="complete")
    event = await wait_for_future_safe(on_entry)

    assert_navigation_info(event, {"context": new_tab["context"], "url": target_url})


async def test_timestamp(bidi_session, current_time, subscribe_events, url, new_tab, wait_for_event, wait_for_future_safe):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE), wait="complete"
    )

    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    time_start = await current_time()

    on_entry = wait_for_event(FRAGMENT_NAVIGATED_EVENT)
    target_url = url(EMPTY_PAGE + '#foo')
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=target_url, wait="complete")
    event = await wait_for_future_safe(on_entry)

    time_end = await current_time()

    assert_navigation_info(
        event,
        {"context": new_tab["context"], "timestamp": int_interval(time_start, time_end)}
    )


async def test_navigation_id(
    bidi_session, new_tab, url, subscribe_events, wait_for_event, wait_for_future_safe
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE), wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT])

    on_fragment_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url(EMPTY_PAGE + '#foo')
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=target_url, wait="complete")

    recursive_compare(
        {
            'context': new_tab["context"],
            'navigation': result["navigation"],
            'timestamp': any_int,
            'url': target_url
        },
        await wait_for_future_safe(on_fragment_navigated),
    )


async def test_url_with_base_tag(bidi_session, subscribe_events, inline, new_tab, wait_for_event, wait_for_future_safe):
    url = inline("""<base href="/relative-path">""")
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=url, wait="complete")

    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    on_fragment_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url + '#foo'
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=target_url, wait="complete")

    recursive_compare(
        {
            'context': new_tab["context"],
            'url': target_url
        },
        await wait_for_future_safe(on_fragment_navigated),
    )


async def test_iframe(
    bidi_session, new_tab, url, inline, subscribe_events, wait_for_event, wait_for_future_safe
):
    initial_url = url(EMPTY_PAGE + '#foo')
    parent_url = inline(f"<iframe src='{initial_url}'></iframe>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=parent_url, wait="complete"
    )
    all_contexts = await bidi_session.browsing_context.get_tree()

    # about:blank + a new tab are top-level contexts.
    assert len(all_contexts) == 2
    parent_info = all_contexts[1]
    assert len(parent_info["children"]) == 1
    child_info = parent_info["children"][0]

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT])

    on_fragment_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url(EMPTY_PAGE + '#bar')
    await bidi_session.browsing_context.navigate(
        context=child_info["context"], url=target_url, wait="complete")

    recursive_compare(
        {
            'context': child_info["context"],
            'timestamp': any_int,
            'url': target_url
        },
        await wait_for_future_safe(on_fragment_navigated),
    )


@pytest.mark.parametrize(
    "hash_before, hash_after",
    [
        ("", "#foo"),
        ("#foo", "#bar"),
        ("#foo", "#foo"),
    ]
)
async def test_document_location(
    bidi_session, new_tab, url, subscribe_events, wait_for_event, wait_for_future_safe, hash_before, hash_after
):
    target_context = new_tab["context"]

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE + hash_before), wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT])

    on_fragment_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url(EMPTY_PAGE + hash_after)

    await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(url) => {
            document.location = url;
        }""",
        arguments=[
            {"type": "string", "value": target_url},
        ],
        await_promise=False,
        target=ContextTarget(target_context),
    )

    recursive_compare(
        {
            'context': target_context,
            'timestamp': any_int,
            'url': target_url
        },
        await wait_for_future_safe(on_fragment_navigated),
    )


@pytest.mark.parametrize(
    "hash_before, hash_after",
    [
        ("", "#foo"),
        ("#foo", "#bar"),
        ("#foo", "#foo"),
    ]
)
async def test_browsing_context_navigate(
    bidi_session, new_tab, url, subscribe_events, wait_for_event, wait_for_future_safe, hash_before, hash_after
):
    target_context = new_tab["context"]

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE + hash_before), wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT])

    on_fragment_navigated = wait_for_event(FRAGMENT_NAVIGATED_EVENT)

    target_url = url(EMPTY_PAGE + hash_after)

    await bidi_session.browsing_context.navigate(
        context=target_context, url=target_url, wait="complete")

    recursive_compare(
        {
            'context': target_context,
            'timestamp': any_int,
            'url': target_url
        },
        await wait_for_future_safe(on_fragment_navigated),
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context(bidi_session, subscribe_events, type_hint):
    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FRAGMENT_NAVIGATED_EVENT, on_event)

    await bidi_session.browsing_context.create(type_hint=type_hint)

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("sandbox", [None, "sandbox_1"])
async def test_document_write(bidi_session, subscribe_events, new_tab, sandbox):
    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FRAGMENT_NAVIGATED_EVENT, on_event)

    await bidi_session.script.evaluate(
        expression="""document.open(); document.write("<h1>Replaced</h1>"); document.close();""",
        target=ContextTarget(new_tab["context"], sandbox),
        await_promise=False
    )

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize(
    "before, after",
    [
        ("", "?foo"),
        ("#foo", ""),
    ]
)
async def test_regular_navigation(bidi_session, subscribe_events, url, new_tab, before, after):
    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=url(EMPTY_PAGE) + before, wait="complete")

    await subscribe_events(events=[FRAGMENT_NAVIGATED_EVENT])

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FRAGMENT_NAVIGATED_EVENT, on_event)

    await bidi_session.browsing_context.navigate(context=new_tab["context"], url=url(EMPTY_PAGE + after), wait="complete")

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()
