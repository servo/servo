import pytest
import random

from tests.support.sync import AsyncPoll

from .. import (
    assert_before_request_sent_event,
    PAGE_EMPTY_TEXT,
    BEFORE_REQUEST_SENT_EVENT,
)

# Note: The cached status cannot be checked in the beforeRequestSent event, but
# the goal is to verify that the events are still emitted for cached requests.


@pytest.mark.asyncio
async def test_cached_document(
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    # `nocache` is not used in cached.py, it is here to avoid the browser cache.
    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )
    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": cached_url}

    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
    )

    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 2

    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
    )


@pytest.mark.asyncio
async def test_page_with_cached_resource(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    # Build a page with a stylesheet resource which will be read from http cache
    # on the next reload.
    # `nocache` is not used in cached.py, it is here to avoid the browser cache.
    cached_css_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&contenttype=text/css&nocache={random.random()}"
    )
    page_with_cached_css = inline(
        f"""
        <head><link rel="stylesheet" type="text/css" href="{cached_css_url}"></head>
        <body>test page with cached stylesheet</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_css,
        wait="complete",
    )

    # Expect two events, one for the document, one for the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    assert_before_request_sent_event(
        events[1],
        expected_request={"method": "GET", "url": cached_css_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events after reload, for the document and the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_before_request_sent_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )

    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": cached_css_url},
    )
