import asyncio

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

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
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
):
    await subscribe_events(events=[FETCH_ERROR_EVENT])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    # Track all received network.beforeRequestSent events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(FETCH_ERROR_EVENT, on_event)

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL))
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
    asyncio.ensure_future(fetch(PAGE_INVALID_URL))
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_iframe_load(
    bidi_session,
    top_context,
    setup_network_test,
    inline,
):
    network_events = await setup_network_test(events=[FETCH_ERROR_EVENT])
    events = network_events[FETCH_ERROR_EVENT]

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(f"<iframe src='{PAGE_INVALID_URL}'></iframe>"),
    )

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 1)

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
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
    top_context,
    wait_for_event,
    url,
    fetch,
    setup_network_test,
    wait_for_future_safe,
):
    await setup_network_test(events=[FETCH_ERROR_EVENT])

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL))
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
        context=top_context["context"],
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
        ("OPTIONS", False),
        ("DELETE", True),
        ("PATCH", True),
        ("PUT", True),
    ],
)
@pytest.mark.asyncio
async def test_request_method(
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    method,
    has_preflight,
):
    network_events = await setup_network_test(events=[FETCH_ERROR_EVENT])
    events = network_events[FETCH_ERROR_EVENT]

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    asyncio.ensure_future(fetch(PAGE_INVALID_URL, method=method))
    await wait_for_future_safe(on_fetch_error)

    assert len(events) == 1

    # Requests which might update the server will fail on the CORS preflight
    # request which uses the OPTIONS method.
    if has_preflight:
        method = "OPTIONS"

    expected_request = {"method": method, "url": PAGE_INVALID_URL}
    assert_fetch_error_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_redirect_fetch(
    bidi_session, wait_for_event, url, fetch, setup_network_test
):
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={PAGE_INVALID_URL}"
    )

    await setup_network_test(
        events=[
            FETCH_ERROR_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    asyncio.ensure_future(fetch(redirect_url))

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
    bidi_session, top_context, wait_for_event, url, setup_network_test
):
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={PAGE_INVALID_URL}"
    )

    await setup_network_test(
        events=[
            FETCH_ERROR_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
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
