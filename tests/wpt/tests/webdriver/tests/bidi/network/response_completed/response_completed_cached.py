import pytest
import random

from tests.support.sync import AsyncPoll

from .. import assert_response_event, PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT


@pytest.mark.asyncio
async def test_cached(
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": cached_url}

    # The first request/response is used to fill the browser cache, so we expect
    # fromCache to be False here.
    expected_response = {
        "url": cached_url,
        "fromCache": False,
        "status": 200,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 2

    # The second request for the same URL has to be read from the local cache.
    expected_response = {
        "url": cached_url,
        "fromCache": True,
        "status": 200,
    }
    assert_response_event(
        events[1],
        expected_request=expected_request,
        expected_response=expected_response,
    )


@pytest.mark.asyncio
async def test_cached_redirect(
    bidi_session,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    text_url = url(PAGE_EMPTY_TEXT)
    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=301&location={text_url}&nocache={random.random()}"
    )

    await fetch(cached_url)

    # Expect two events, one for the initial request and one for the redirect.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    # The first request/response is used to fill the cache, so we expect
    # fromCache to be False here.
    expected_request = {"method": "GET", "url": cached_url}
    expected_response = {
        "url": cached_url,
        "fromCache": False,
        "status": 301,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    # The second request is the redirect
    redirected_request = {"method": "GET", "url": text_url}
    redirected_response = {"url": text_url, "status": 200}
    assert_response_event(
        events[1],
        expected_request=redirected_request,
        expected_response=redirected_response,
    )

    await fetch(cached_url)
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    # The third request hits cached_url again and has to be read from the local cache.
    expected_response = {
        "url": cached_url,
        "fromCache": True,
        "status": 301,
    }
    assert_response_event(
        events[2],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    # The fourth request is the redirect
    assert_response_event(
        events[3],
        expected_request=redirected_request,
        expected_response=redirected_response,
    )


@pytest.mark.asyncio
async def test_cached_revalidate(
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    # `nocache` is not used in cached.py, it is here to avoid the browser cache.
    revalidate_url = url(
        f"/webdriver/tests/support/http_handlers/must-revalidate.py?nocache={random.random()}"
    )
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(revalidate_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": revalidate_url}
    expected_response = {
        "url": revalidate_url,
        "fromCache": False,
        "status": 200,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    # Note that we pass a specific header so that the must-revalidate.py handler
    # can decide to return a 304 without having to use another URL.
    await fetch(revalidate_url, headers={"return-304": "true"})
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 2

    # Here fromCache should still be false, because for a 304 response the response
    # cache state is "validated" and fromCache is only true if cache state is "local"
    expected_response = {
        "url": revalidate_url,
        "fromCache": False,
        "status": 304,
    }
    assert_response_event(
        events[1],
        expected_request=expected_request,
        expected_response=expected_response,
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
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    # Expect two events, one for the page, one for the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    assert_response_event(
        events[1],
        expected_request={"method": "GET", "url": cached_css_url},
        expected_response={"url": cached_css_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two additional events after reload, for the page and the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_response_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )

    assert_response_event(
        events[3],
        expected_request={"method": "GET", "url": cached_css_url},
        expected_response={"url": cached_css_url, "fromCache": True},
    )
