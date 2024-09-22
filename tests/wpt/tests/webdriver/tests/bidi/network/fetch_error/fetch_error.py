import asyncio

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from ... import number_interval
from .. import (
    assert_fetch_error_event,
    assert_response_event,
    FETCH_ERROR_EVENT,
    PAGE_EMPTY_HTML,
    RESPONSE_COMPLETED_EVENT,
    PAGE_INVALID_URL,
)


@pytest.mark.asyncio
async def test_subscribe_status(
    bidi_session,
    subscribe_events,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
):
    await subscribe_events(events=[FETCH_ERROR_EVENT])

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    # Track all received network.beforeRequestSent events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FETCH_ERROR_EVENT, on_event)

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL, context=new_tab))
    await wait_for_future_safe(on_fetch_error)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )

    await bidi_session.session.unsubscribe(events=[FETCH_ERROR_EVENT])

    # Fetch the invalid url again, with an additional parameter to bypass the
    # cache and check no new event is received.
    asyncio.ensure_future(fetch(PAGE_INVALID_URL, context=new_tab))
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_aborted_request(
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    setup_network_test,
    url,
    fetch,
):
    network_events = await setup_network_test(
        events=[FETCH_ERROR_EVENT], context=new_tab["context"]
    )
    events = network_events[FETCH_ERROR_EVENT]

    # Prepare a slow url
    slow_url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.txt?pipe=trickle(d10)"
    )
    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(
        fetch(PAGE_INVALID_URL, context=new_tab, timeout_in_seconds=0)
    )
    fetch_error_event = await wait_for_future_safe(on_fetch_error)


@pytest.mark.asyncio
async def test_iframe_load(
    bidi_session,
    new_tab,
    setup_network_test,
    inline,
):
    network_events = await setup_network_test(
        events=[FETCH_ERROR_EVENT], context=new_tab["context"]
    )
    events = network_events[FETCH_ERROR_EVENT]

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(f"<iframe src='{PAGE_INVALID_URL}'></iframe>"),
    )

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 1)

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    frame_context = contexts[0]["children"][0]

    assert len(events) == 1
    assert_fetch_error_event(
        events[0],
        expected_request={"url": PAGE_INVALID_URL},
        context=frame_context["context"],
    )


@pytest.mark.asyncio
async def test_navigation_id(
    bidi_session,
    new_tab,
    wait_for_event,
    url,
    fetch,
    setup_network_test,
    wait_for_future_safe,
):
    await setup_network_test(events=[FETCH_ERROR_EVENT], context=new_tab["context"])

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL, context=new_tab))
    fetch_error_event = await wait_for_future_safe(on_fetch_error)

    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        fetch_error_event,
        expected_request=expected_request,
    )
    # Check that requests not related to a navigation have no navigation id.
    assert fetch_error_event["navigation"] is None

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=PAGE_INVALID_URL,
    )
    fetch_error_event = await wait_for_future_safe(on_fetch_error)

    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        fetch_error_event,
        expected_request=expected_request,
        navigation=result["navigation"],
    )
    assert fetch_error_event["navigation"] == result["navigation"]


@pytest.mark.parametrize(
    "method, has_preflight",
    [
        ("GET", False),
        ("HEAD", False),
        ("POST", False),
        ("OPTIONS", True),
        ("DELETE", True),
        ("PATCH", True),
        ("PUT", True),
    ],
)
@pytest.mark.asyncio
async def test_request_method(
    bidi_session,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    method,
    has_preflight,
):
    network_events = await setup_network_test(
        events=[FETCH_ERROR_EVENT], context=new_tab["context"]
    )
    events = network_events[FETCH_ERROR_EVENT]

    asyncio.ensure_future(fetch(PAGE_INVALID_URL, context=new_tab, method=method))

    # Requests which might update the server will also fail the CORS preflight
    # request which uses the OPTIONS method.
    expected_events = 2 if has_preflight else 1

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= expected_events)
    assert len(events) == expected_events

    # TODO: At the moment the event order for preflight requests differs between
    # Chrome and Firefox so we cannot assume the order of fetchError events.
    # See https://bugzilla.mozilla.org/show_bug.cgi?id=1879402.

    # Check that fetch_error events have the expected methods.
    assert method in [e["request"]["method"] for e in events]
    if has_preflight:
        assert "OPTIONS" in [e["request"]["method"] for e in events]

    for event in events:
        assert_fetch_error_event(
            event,
            expected_request={"url": PAGE_INVALID_URL},
        )


@pytest.mark.asyncio
async def test_request_timing_info(
    bidi_session,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
    current_time,
):
    network_events = await setup_network_test(
        events=[FETCH_ERROR_EVENT], context=new_tab["context"]
    )
    events = network_events[FETCH_ERROR_EVENT]

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL, context=new_tab))
    await wait_for_future_safe(on_fetch_error)

    time_end = await current_time()
    time_range = number_interval(time_start, time_end)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        events[0],
        expected_request=expected_request,
        expected_time_range=time_range,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_redirect_fetch(
    bidi_session, new_tab, wait_for_event, url, fetch, setup_network_test
):
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={PAGE_INVALID_URL}"
    )

    await setup_network_test(
        events=[
            FETCH_ERROR_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        context=new_tab["context"],
    )

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    asyncio.ensure_future(fetch(redirect_url, context=new_tab))

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    fetch_error_event = await on_fetch_error
    response_completed_event = await on_response_completed

    expected_request = {"method": "GET", "url": redirect_url}
    assert_response_event(
        response_completed_event,
        expected_request=expected_request,
        redirect_count=0,
    )
    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        fetch_error_event, expected_request=expected_request, redirect_count=1
    )

    # Check that both requests share the same requestId
    assert (
        fetch_error_event["request"]["request"]
        == response_completed_event["request"]["request"]
    )


@pytest.mark.asyncio
async def test_redirect_navigation(
    bidi_session, new_tab, wait_for_event, url, setup_network_test
):
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={PAGE_INVALID_URL}"
    )

    await setup_network_test(
        events=[
            FETCH_ERROR_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ],
        context=new_tab["context"],
    )

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=redirect_url,
    )

    wait = AsyncPoll(bidi_session, timeout=2)
    fetch_error_event = await on_fetch_error
    response_completed_event = await on_response_completed

    expected_request = {"method": "GET", "url": redirect_url}
    assert_response_event(
        response_completed_event,
        expected_request=expected_request,
        navigation=result["navigation"],
        redirect_count=0,
    )
    expected_request = {"method": "GET", "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        fetch_error_event,
        expected_request=expected_request,
        navigation=result["navigation"],
        redirect_count=1,
    )

    # Check that all events share the same requestId
    assert (
        fetch_error_event["request"]["request"]
        == response_completed_event["request"]["request"]
    )
