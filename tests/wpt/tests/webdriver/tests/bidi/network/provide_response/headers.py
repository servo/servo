import pytest

from webdriver.bidi.modules.network import (
    Header,
    NetworkStringValue,
    SetCookieHeader,
)

from .. import (
    assert_response_event,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)

from ... import recursive_compare

pytestmark = pytest.mark.asyncio

LOAD_EVENT = "browsingContext.load"


@pytest.mark.parametrize(
    "headers",
    [
        {},
        {"a": "1"},
        {"a": "1", "b": "2"},
    ],
)
async def test_headers_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
    headers,
):
    request = await setup_blocked_request(phase="beforeRequestSent")

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            RESPONSE_STARTED_EVENT,
        ]
    )

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    response_headers = []
    for name, value in headers.items():
        response_headers.append(Header(name=name, value=NetworkStringValue(value)))

    await bidi_session.network.provide_response(
        request=request,
        body=NetworkStringValue("overridden response"),
        status_code=200,
        reason_phrase="OK",
        headers=response_headers,
    )

    response_started_event = await wait_for_future_safe(on_response_started)
    assert_response_event(
        response_started_event, expected_response={"headers": response_headers}
    )
    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event, expected_response={"headers": response_headers}
    )


async def test_set_cookie_header_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
):
    request = await setup_blocked_request(
        phase="beforeRequestSent",
        navigate=True,
    )

    await subscribe_events(events=[LOAD_EVENT])
    on_load = wait_for_event(LOAD_EVENT)

    response_header = Header(
        name="Set-Cookie", value=NetworkStringValue("aaa=bbb;Path=/")
    )

    await bidi_session.network.provide_response(
        request=request,
        body=NetworkStringValue("overridden response"),
        status_code=200,
        reason_phrase="OK",
        headers=[response_header],
    )

    await wait_for_future_safe(on_load)

    cookies = await bidi_session.storage.get_cookies()
    assert len(cookies["cookies"]) == 1

    cookie = cookies["cookies"][0]

    expected_cookie = {
        "httpOnly": False,
        "name": "aaa",
        "path": "/",
        "sameSite": "none",
        "secure": False,
        "size": 6,
        "value": {"type": "string", "value": "bbb"},
    }
    recursive_compare(expected_cookie, cookie)

    await bidi_session.storage.delete_cookies()


# Check that cookies from Set-Cookie headers of the headers parameter
# and from the cookies parameter are both present in the response.
async def test_set_cookie_header_and_cookies_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
):
    request = await setup_blocked_request(
        phase="beforeRequestSent",
        navigate=True,
    )

    await subscribe_events(events=[LOAD_EVENT])
    on_load = wait_for_event(LOAD_EVENT)

    response_header = Header(
        name="Set-Cookie", value=NetworkStringValue("foo=bar;Path=/")
    )
    response_cookie = SetCookieHeader(
        name="baz", value=NetworkStringValue("biz"), path="/"
    )

    await bidi_session.network.provide_response(
        request=request,
        body=NetworkStringValue("overridden response"),
        status_code=200,
        reason_phrase="OK",
        headers=[response_header],
        cookies=[response_cookie],
    )

    await wait_for_future_safe(on_load)

    cookies = await bidi_session.storage.get_cookies()
    assert len(cookies["cookies"]) == 2

    if cookies["cookies"][0]["name"] == "foo":
        cookie_from_headers_param = cookies["cookies"][0]
        cookie_from_cookies_param = cookies["cookies"][1]
    else:
        cookie_from_headers_param = cookies["cookies"][1]
        cookie_from_cookies_param = cookies["cookies"][0]

    expected_cookie_from_headers_param = {
        "httpOnly": False,
        "name": "foo",
        "path": "/",
        "sameSite": "none",
        "secure": False,
        "size": 6,
        "value": {"type": "string", "value": "bar"},
    }
    recursive_compare(expected_cookie_from_headers_param, cookie_from_headers_param)

    expected_cookie_from_cookies_param = {
        "httpOnly": False,
        "name": "baz",
        "path": "/",
        "sameSite": "none",
        "secure": False,
        "size": 6,
        "value": {"type": "string", "value": "biz"},
    }
    recursive_compare(expected_cookie_from_cookies_param, cookie_from_cookies_param)

    await bidi_session.storage.delete_cookies()
