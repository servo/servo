import asyncio

import pytest

from webdriver.bidi.modules.script import ContextTarget

from tests.support.sync import AsyncPoll

from ... import number_interval
from .. import (
    assert_response_event,
    HTTP_STATUS_AND_STATUS_TEXT,
    PAGE_DATA_URL_HTML,
    PAGE_DATA_URL_IMAGE,
    PAGE_EMPTY_HTML,
    PAGE_EMPTY_IMAGE,
    PAGE_EMPTY_SCRIPT,
    PAGE_EMPTY_SVG,
    PAGE_EMPTY_TEXT,
    PAGE_INITIATOR,
    PAGE_SERVICEWORKER_HTML,
    RESPONSE_STARTED_EVENT,
)


@pytest.mark.asyncio
async def test_subscribe_status(bidi_session, subscribe_events, top_context, wait_for_event, wait_for_future_safe, url, fetch):
    await subscribe_events(events=[RESPONSE_STARTED_EVENT])

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    # Track all received network.responseStarted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(RESPONSE_STARTED_EVENT, on_event)

    text_url = url(PAGE_EMPTY_TEXT)
    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(text_url)
    await wait_for_future_safe(on_response_started)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": text_url}
    expected_response = {
        "url": text_url,
        "fromCache": False,
        "mimeType": "text/plain",
        "status": 200,
        "statusText": "OK",
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )

    await bidi_session.session.unsubscribe(events=[RESPONSE_STARTED_EVENT])

    # Fetch the text url again, with an additional parameter to bypass the cache
    # and check no new event is received.
    await fetch(f"{text_url}?nocache")
    await asyncio.sleep(0.5)
    assert len(events) == 1

    remove_listener()


@pytest.mark.asyncio
async def test_iframe_load(
    bidi_session,
    top_context,
    setup_network_test,
    test_page,
    test_page_same_origin_frame,
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page_same_origin_frame,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context = contexts[0]["children"][0]

    assert len(events) == 2
    assert_response_event(
        events[0],
        expected_request={"url": test_page_same_origin_frame},
        context=top_context["context"],
    )
    assert_response_event(
        events[1],
        expected_request={"url": test_page},
        context=frame_context["context"],
    )


@pytest.mark.asyncio
async def test_load_page_twice(
    bidi_session, top_context, wait_for_event, wait_for_future_safe, url, setup_network_test
):
    html_url = url(PAGE_EMPTY_HTML)

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=html_url,
        wait="complete",
    )
    await wait_for_future_safe(on_response_started)

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
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        navigation=result["navigation"],
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_request_bodysize(
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    html_url = url(PAGE_EMPTY_HTML)

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_before_request_sent = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(html_url, method="POST", post_data="{'a': 1}")
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {
        "method": "POST",
        "url": html_url,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )
    assert events[0]["request"]["bodySize"] > 0


@pytest.mark.asyncio
async def test_request_timing_info(
    url,
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    current_time,
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(url(PAGE_EMPTY_HTML), method="GET")
    await wait_for_future_safe(on_response_started)

    time_end = await current_time()
    time_range = number_interval(time_start - 1, time_end + 1)

    assert len(events) == 1

    expected_request = {
        "method": "GET",
        "url": url(PAGE_EMPTY_HTML),
    }
    expected_response = {"url": url(PAGE_EMPTY_HTML)}
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        expected_time_range=time_range,
        redirect_count=0,
    )


@pytest.mark.parametrize(
    "status, status_text",
    HTTP_STATUS_AND_STATUS_TEXT,
)
@pytest.mark.asyncio
async def test_response_status(
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test, status, status_text
):
    status_url = url(
        f"/webdriver/tests/support/http_handlers/status.py?status={status}&nocache={RESPONSE_STARTED_EVENT}"
    )

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(status_url)
    await wait_for_future_safe(on_response_started)

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
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_response_headers(wait_for_event, wait_for_future_safe, url, fetch, setup_network_test):
    headers_url = url(
        "/webdriver/tests/support/http_handlers/headers.py?header=foo:bar&header=baz:biz"
    )

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(headers_url, method="GET")
    await wait_for_future_safe(on_response_started)

    assert len(events) == 1

    expected_request = {"method": "GET", "url": headers_url}
    expected_response = {
        "url": headers_url,
        "fromCache": False,
        "mimeType": "text/plain",
        "status": 200,
        "statusText": "OK",
        "headers": (
            {"name": "foo", "value": {"type": "string", "value": "bar"}},
            {"name": "baz", "value": {"type": "string", "value": "biz"}},
        ),
        "protocol": "http/1.1",
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
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
    url, wait_for_event, wait_for_future_safe, fetch, setup_network_test, page_url, mime_type
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(url(page_url), method="GET")
    await wait_for_future_safe(on_response_started)

    assert len(events) == 1

    expected_request = {"method": "GET", "url": url(page_url)}
    expected_response = {"url": url(page_url), "mimeType": mime_type}
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_www_authenticate(
    bidi_session, url, fetch, new_tab, wait_for_event, wait_for_future_safe, setup_network_test
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )
    auth_url = url(
        "/webdriver/tests/support/http_handlers/authentication.py?realm=testrealm"
    )

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    asyncio.ensure_future(fetch(url=auth_url, context=new_tab))

    await wait_for_future_safe(on_response_started)

    assert len(events) == 1

    expected_request = {"method": "GET", "url": auth_url}
    expected_response = {
        "url": auth_url,
        "authChallenges": [
            ({"scheme": "Basic", "realm": "testrealm"}),
        ],
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_redirect(bidi_session, url, fetch, setup_network_test):
    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}"
    )

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    await fetch(redirect_url, method="GET")

    # Wait until we receive two events, one for the initial request and one for
    # the redirection.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    expected_request = {"method": "GET", "url": redirect_url}
    assert_response_event(
        events[0],
        expected_request=expected_request,
        redirect_count=0,
    )
    expected_request = {"method": "GET", "url": text_url}
    assert_response_event(
        events[1], expected_request=expected_request, redirect_count=1
    )

    # Check that both requests share the same requestId
    assert events[0]["request"]["request"] == events[1]["request"]["request"]


@pytest.mark.asyncio
async def test_serviceworker_request(
    bidi_session,
    new_tab,
    url,
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    current_time,
):
    serviceworker_url = url(PAGE_SERVICEWORKER_HTML)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=serviceworker_url,
        wait="complete",
    )

    await bidi_session.script.evaluate(
        expression="registerServiceWorker()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    # Make a request to serviceworker_url via fetch on the page, but any url
    # would work here as this should be intercepted by the serviceworker.
    await fetch(serviceworker_url, context=new_tab, method="GET")
    await wait_for_future_safe(on_response_started)

    time_end = await current_time()
    time_range = number_interval(time_start - 1, time_end + 1)

    assert len(events) == 1

    assert_response_event(
        events[0],
        expected_request={
            "method": "GET",
            "url": serviceworker_url,
        },
        expected_response={
            "url": serviceworker_url,
            "statusText": "OK from serviceworker",
        },
        expected_time_range=time_range,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_url_with_fragment(
    url, wait_for_event, wait_for_future_safe, fetch, setup_network_test, current_time
):
    fragment_url = url(f"{PAGE_EMPTY_HTML}#foo")

    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    await fetch(fragment_url, method="GET")
    await wait_for_future_safe(on_response_started)

    time_end = await current_time()
    time_range = number_interval(time_start - 1, time_end + 1)

    assert len(events) == 1

    # Assert that the event contains the full fragment URL both in requestData
    # and responseData
    assert_response_event(
        events[0],
        expected_request={
            "method": "GET",
            "url": fragment_url,
        },
        expected_response={"url": fragment_url},
        expected_time_range=time_range,
        redirect_count=0,
    )


@pytest.mark.parametrize(
    "page_url, mimeType",
    [(PAGE_DATA_URL_HTML, "text/html"), (PAGE_DATA_URL_IMAGE, "image/png")],
    ids=["html", "image"],
)
@pytest.mark.asyncio
async def test_navigate_data_url(
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    setup_network_test,
    page_url,
    mimeType,
    current_time,
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page_url, wait="complete"
    )
    await wait_for_future_safe(on_response_started)

    time_end = await current_time()
    time_range = number_interval(time_start - 1, time_end + 1)

    assert len(events) == 1

    assert_response_event(
        events[0],
        expected_request={
            "method": "GET",
            "url": page_url,
        },
        expected_response={
            "headers": [
                {"name": "Content-Type", "value": {"type": "string", "value": mimeType}}
            ],
            "mimeType": mimeType,
            "protocol": "data",
            "status": 200,
            "statusText": "OK",
            "url": page_url,
        },
        expected_time_range=time_range,
        redirect_count=0,
        navigation=result["navigation"],
    )
    assert events[0]["navigation"] is not None


@pytest.mark.parametrize(
    "fetch_url, mimeType",
    [(PAGE_DATA_URL_HTML, "text/html"), (PAGE_DATA_URL_IMAGE, "image/png")],
    ids=["html", "image"],
)
@pytest.mark.asyncio
async def test_fetch_data_url(
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    fetch_url,
    mimeType,
    current_time,
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(fetch_url, method="GET")
    await wait_for_future_safe(on_response_started)

    time_end = await current_time()
    time_range = number_interval(time_start - 1, time_end + 1)

    assert len(events) == 1

    assert_response_event(
        events[0],
        expected_request={
            "method": "GET",
            "url": fetch_url,
        },
        expected_response={
            "headers": [
                {"name": "Content-Type", "value": {"type": "string", "value": mimeType}}
            ],
            "mimeType": mimeType,
            "protocol": "data",
            "status": 200,
            "statusText": "OK",
            "url": fetch_url,
        },
        expected_time_range=time_range,
        redirect_count=0,
    )
    assert events[0]["navigation"] is None


@pytest.mark.asyncio
async def test_destination_initiator(
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    fetch,
    setup_network_test,
    url,
):
    network_events = await setup_network_test(events=[RESPONSE_STARTED_EVENT])
    events = network_events[RESPONSE_STARTED_EVENT]

    page_url = url(PAGE_INITIATOR["HTML"])

    result = await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page_url, wait="complete"
    )

    assert len(events) == 6

    def assert_initiator_destination(url, initiator_type, destination):
        event = next(e for e in events if url in e["request"]["url"])
        assert_response_event(
            event,
            expected_request={
                "destination": destination,
                "initiatorType": initiator_type,
            },
        )

    assert_initiator_destination(PAGE_INITIATOR["HTML"], None, "")
    assert_initiator_destination(PAGE_INITIATOR["SCRIPT"], "script", "script")
    assert_initiator_destination(PAGE_INITIATOR["STYLESHEET"], "link", "style")
    assert_initiator_destination(PAGE_INITIATOR["IMAGE"], "img", "image")
    assert_initiator_destination(PAGE_INITIATOR["BACKGROUND"], "css", "image")
    assert_initiator_destination(PAGE_EMPTY_HTML, "iframe", "iframe")

    # Perform an additional fetch, and check its destination.
    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    await fetch(page_url, method="GET")
    event = await wait_for_future_safe(on_response_started)

    assert_response_event(
        event,
        expected_request={
            "destination": "",
            "initiatorType": "fetch",
        },
    )
