import asyncio
import json

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from ... import any_int
from .. import assert_response_started_event

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_IMAGE = "/webdriver/tests/bidi/network/support/empty.png"
PAGE_EMPTY_SCRIPT = "/webdriver/tests/bidi/network/support/empty.js"
PAGE_EMPTY_SVG = "/webdriver/tests/bidi/network/support/empty.svg"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"

# The following tests are marked as tentative until
# https://github.com/w3c/webdriver-bidi/pull/204 is merged.


@pytest.mark.asyncio
async def test_subscribe_status(bidi_session, top_context, wait_for_event, url, fetch):
    await bidi_session.session.subscribe(events=["network.responseStarted"])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    # Track all received network.responseStarted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        "network.responseStarted", on_event
    )

    text_url = url(PAGE_EMPTY_TEXT)
    on_response_started = wait_for_event("network.responseStarted")
    await fetch(text_url)
    await on_response_started

    assert len(events) == 1
    expected_request = {"method": "GET", "url": text_url}
    expected_response = {
        "url": text_url,
        "fromCache": False,
        "mimeType": "text/plain",
        "status": 200,
        "statusText": "OK",
    }
    assert_response_started_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
        is_redirect=False,
    )

    await bidi_session.session.unsubscribe(events=["network.responseStarted"])

    # Fetch the text url again, with an additional parameter to bypass the cache
    # and check no new event is received.
    await fetch(f"{text_url}?nocache")
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_load_page_twice(
    bidi_session, top_context, wait_for_event, url, fetch, setup_network_test
):
    html_url = url(PAGE_EMPTY_HTML)

    network_events = await setup_network_test(events=["network.responseStarted"])
    events = network_events["network.responseStarted"]

    on_response_started = wait_for_event("network.responseStarted")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await on_response_started

    assert len(events) == 1
    expected_request = {"method": "GET", "url": html_url}
    expected_response = {
        "url": html_url,
        "fromCache": False,
        "mimeType": "text/html",
        "status": 200,
        "statusText": "OK",
        "protocol": "http/1.1",
    }
    assert_response_started_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
        is_redirect=False,
    )


@pytest.mark.parametrize(
    "status, status_text",
    [
        (101, "Switching Protocols"),
        (200, "OK"),
        (201, "Created"),
        (202, "Accepted"),
        (203, "Non-Authoritative Information"),
        (204, "No Content"),
        (205, "Reset Content"),
        (206, "Partial Content"),
        (300, "Multiple Choices"),
        (301, "Moved Permanently"),
        (302, "Found"),
        (303, "See Other"),
        (304, "Not Modified"),
        (305, "Use Proxy"),
        (307, "Temporary Redirect"),
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (402, "Payment Required"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (405, "Method Not Allowed"),
        (406, "Not Acceptable"),
        (407, "Proxy Authentication Required"),
        (408, "Request Timeout"),
        (409, "Conflict"),
        (410, "Gone"),
        (411, "Length Required"),
        (412, "Precondition Failed"),
        (415, "Unsupported Media Type"),
        (417, "Expectation Failed"),
        (500, "Internal Server Error"),
        (501, "Not Implemented"),
        (502, "Bad Gateway"),
        (503, "Service Unavailable"),
        (504, "Gateway Timeout"),
        (505, "HTTP Version Not Supported"),
    ],
)
@pytest.mark.asyncio
async def test_response_status(
    bidi_session, wait_for_event, url, fetch, setup_network_test, status, status_text
):
    status_url = url(f"/webdriver/tests/support/http_handlers/status.py?status={status}")

    network_events = await setup_network_test(events=["network.responseStarted"])
    events = network_events["network.responseStarted"]

    on_response_started = wait_for_event("network.responseStarted")
    await fetch(status_url)
    await on_response_started

    assert len(events) == 1
    expected_request = {"method": "GET", "url": status_url}
    expected_response = {
        "url": status_url,
        "fromCache": False,
        "mimeType": "text/plain",
        "status": status,
        "statusText": status_text,
        "protocol": "http/1.1",
    }
    assert_response_started_event(
        events[0],
        expected_response=expected_response,
        redirect_count=0,
        is_redirect=False,
    )


@pytest.mark.asyncio
async def test_response_headers(
    bidi_session, wait_for_event, url, fetch, setup_network_test
):
    headers_url = url(
        "/webdriver/tests/support/http_handlers/headers.py?header=foo:bar&header=baz:biz"
    )

    network_events = await setup_network_test(events=["network.responseStarted"])
    events = network_events["network.responseStarted"]

    on_response_started = wait_for_event("network.responseStarted")
    await fetch(headers_url, method="GET")
    await on_response_started

    assert len(events) == 1

    expected_request = {"method": "GET", "url": headers_url}
    expected_response = {
        "url": headers_url,
        "fromCache": False,
        "mimeType": "text/plain",
        "status": 200,
        "statusText": "OK",
        "headers": (
            {"name": "foo", "value": "bar"},
            {"name": "baz", "value": "biz"},
        ),
        "protocol": "http/1.1",
    }
    assert_response_started_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
        is_redirect=False,
    )


@pytest.mark.parametrize(
    "page_url, mime_type",
    [
        (PAGE_EMPTY_HTML, "text/html"),
        (PAGE_EMPTY_TEXT, "text/plain"),
        (PAGE_EMPTY_SCRIPT, "text/javascript"),
        (PAGE_EMPTY_IMAGE, "image/png"),
        (PAGE_EMPTY_SVG, "image/svg+xml"),
    ],
)
@pytest.mark.asyncio
async def test_response_mime_type_file(
    bidi_session, url, wait_for_event, fetch, setup_network_test, page_url, mime_type
):
    network_events = await setup_network_test(events=["network.responseStarted"])
    events = network_events["network.responseStarted"]

    on_response_started = wait_for_event("network.responseStarted")
    await fetch(url(page_url), method="GET")
    await on_response_started

    assert len(events) == 1

    expected_request = {"method": "GET", "url": url(page_url)}
    expected_response = {"url": url(page_url), "mimeType": mime_type}
    assert_response_started_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
        is_redirect=False,
    )


@pytest.mark.asyncio
async def test_redirect(bidi_session, wait_for_event, url, fetch, setup_network_test):
    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}")

    network_events = await setup_network_test(events=["network.responseStarted"])
    events = network_events["network.responseStarted"]

    await fetch(redirect_url, method="GET")

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    expected_request = {"method": "GET", "url": redirect_url}
    assert_response_started_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
        is_redirect=False,
    )
    expected_request = {"method": "GET", "url": text_url}
    assert_response_started_event(
        events[1], expected_request=expected_request, redirect_count=1, is_redirect=True
    )

    # Check that both requests share the same requestId
    assert events[0]["request"]["request"] == events[1]["request"]["request"]
