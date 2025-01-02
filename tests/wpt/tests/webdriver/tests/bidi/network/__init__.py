import base64
import random
import urllib
from datetime import datetime, timedelta, timezone

from webdriver.bidi.modules.network import (
    NetworkStringValue,
    SetCookieHeader,
)

from .. import (
    any_bool,
    any_dict,
    any_int,
    any_number,
    any_int_or_null,
    any_list,
    any_string,
    any_string_or_null,
    assert_cookies,
    int_interval,
    recursive_compare,
)



def assert_bytes_value(bytes_value):
    assert bytes_value["type"] in ["string", "base64"]
    any_string(bytes_value["value"])


def assert_headers(event_headers, expected_headers):
    # The browser sets request headers, only assert that the expected headers
    # are included in the request's headers.
    assert len(event_headers) >= len(expected_headers)
    for header in expected_headers:
        assert next(h for h in event_headers if header == h) is not None


def assert_timing_info(timing_info, expected_time_range=None):
    # First assert time origin, which is reused to assert the following values.
    time_origin = timing_info.get("timeOrigin")
    any_number(time_origin)

    def assert_timing(actual):
        # Check that the timing is a number
        any_number(actual)

        # If a time range was provided, assert that the time is within the
        # provided bounds.
        # Unless timing is 0, which means the timing is not relevant for the
        # current network event, or is not known yet.
        if expected_time_range is not None and actual != 0:
            # Add time_origin to actual to get the absolute time corresponding
            # to the timing.
            expected_time_range(actual + time_origin)

    # Assert all other timings.
    recursive_compare(
        {
            "requestTime": assert_timing,
            "redirectStart": assert_timing,
            "redirectEnd": assert_timing,
            "fetchStart": assert_timing,
            "dnsStart": assert_timing,
            "dnsEnd": assert_timing,
            "connectStart": assert_timing,
            "connectEnd": assert_timing,
            "tlsStart": assert_timing,
            "requestStart": assert_timing,
            "responseStart": assert_timing,
            "responseEnd": assert_timing,
        },
        timing_info,
    )


def assert_request_data(request_data, expected_request, expected_time_range):
    recursive_compare(
        {
            "bodySize": any_int_or_null,
            "cookies": any_list,
            "destination": any_string,
            "headers": any_list,
            "headersSize": any_int,
            "initiatorType": any_string_or_null,
            "method": any_string,
            "request": any_string,
            "timings": any_dict,
            "url": any_string,
        },
        request_data,
    )

    for cookie in request_data["cookies"]:
        assert_bytes_value(cookie["value"])

    if "cookies" in expected_request:
        assert_cookies(request_data["cookies"], expected_request["cookies"])
        # While recursive_compare tolerates missing entries in dict, arrays
        # need to have the exact same number of items, and be in the same order.
        # We don't want to assert all headers and cookies, so we do a custom
        # assert for each and then delete it before using recursive_compare.
        del expected_request["cookies"]

    for header in request_data["headers"]:
        assert_bytes_value(header["value"])

    if "headers" in expected_request:
        assert_headers(request_data["headers"], expected_request["headers"])
        # Remove headers before using recursive_compare, see comment for cookies
        del expected_request["headers"]

    assert_timing_info(request_data["timings"], expected_time_range)

    recursive_compare(expected_request, request_data)


def assert_base_parameters(
    event,
    context=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
    expected_time_range=None,
):
    recursive_compare(
        {
            "context": any_string_or_null,
            "isBlocked": any_bool,
            "navigation": any_string_or_null,
            "redirectCount": any_int,
            "request": any_dict,
            "timestamp": any_int,
        },
        event,
    )

    if context is not None:
        assert event["context"] == context

    if is_blocked is not None:
        assert event["isBlocked"] == is_blocked

    if event["isBlocked"]:
        assert isinstance(event["intercepts"], list)
        assert len(event["intercepts"]) > 0
        for intercept in event["intercepts"]:
            assert isinstance(intercept, str)
    else:
        assert "intercepts" not in event

    if intercepts is not None:
        assert event["intercepts"] == intercepts

    if navigation is not None:
        assert event["navigation"] == navigation

    if redirect_count is not None:
        assert event["redirectCount"] == redirect_count

    # Assert request data (expected_time_range is optional)
    if expected_request is not None:
        assert_request_data(event["request"], expected_request, expected_time_range)


def assert_before_request_sent_event(
    event,
    context=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
    expected_time_range=None,
):
    # Assert initiator
    if "initiator" in event:
        assert isinstance(event["initiator"], dict)

    # Assert base parameters
    assert_base_parameters(
        event,
        context=context,
        intercepts=intercepts,
        is_blocked=is_blocked,
        navigation=navigation,
        redirect_count=redirect_count,
        expected_request=expected_request,
        expected_time_range=expected_time_range,
    )


def assert_fetch_error_event(
    event,
    context=None,
    errorText=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
    expected_time_range=None,
):
    # Assert errorText
    assert isinstance(event["errorText"], str)

    if errorText is not None:
        assert event["errorText"] == errorText

    # Assert base parameters
    assert_base_parameters(
        event,
        context=context,
        intercepts=intercepts,
        is_blocked=is_blocked,
        navigation=navigation,
        redirect_count=redirect_count,
        expected_request=expected_request,
        expected_time_range=expected_time_range,
    )


def assert_response_data(response_data, expected_response):
    recursive_compare(
        {
            "bodySize": any_int_or_null,
            "bytesReceived": any_int,
            "content": {
                "size": any_int_or_null,
            },
            "fromCache": any_bool,
            "headersSize": any_int_or_null,
            "protocol": any_string,
            "status": any_int,
            "statusText": any_string,
            "url": any_string,
        },
        response_data,
    )

    for header in response_data["headers"]:
        assert_bytes_value(header["value"])

    for header in response_data["headers"]:
        assert_bytes_value(header["value"])

    if "headers" in expected_response:
        assert_headers(response_data["headers"], expected_response["headers"])
        # Remove headers before using recursive_compare, see comment for cookies
        # in assert_request_data
        del expected_response["headers"]

    if response_data["status"] in [401, 407]:
        assert isinstance(response_data["authChallenges"], list)
    else:
        assert "authChallenges" not in response_data

    recursive_compare(expected_response, response_data)


def assert_response_event(
    event,
    context=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
    expected_response=None,
    expected_time_range=None,
):
    # Assert response data
    any_dict(event["response"])
    if expected_response is not None:
        assert_response_data(event["response"], expected_response)

    # Assert base parameters
    assert_base_parameters(
        event,
        context=context,
        intercepts=intercepts,
        is_blocked=is_blocked,
        navigation=navigation,
        redirect_count=redirect_count,
        expected_request=expected_request,
        expected_time_range=expected_time_range,
    )


# Create a simple cookie or set-cookie header. They share the same structure
# as a regular header, so this is simple alias for create_header.
def create_cookie_header(overrides=None, value_overrides=None):
    return create_header(overrides, value_overrides)


# Create a simple header dict, with mandatory name and value keys.
# Use the `overrides` argument to update the values of those properties, or to
# add new top-level keys.
# Use the `value_overrides` argument to update keys nested in the `value` dict.
def create_header(overrides=None, value_overrides=None):
    header = {
        "name": "test",
        "value": {
            "type": "string",
            "value": "foo"
        }
    }

    if overrides is not None:
        header.update(overrides)

    if value_overrides is not None:
        header["value"].update(value_overrides)

    return header


def get_cached_url(content_type, response):
    """
    Build a URL for a resource which will be fully cached.

    :param content_type: Response content type eg "text/css".
    :param response: Response body>

    :return: Relative URL as a string, typically should be used with the
        `url` fixture.
    """
    # `nocache` is not used in cached.py, it is here to bypass the browser cache
    # from previous tests accessing the same URL.
    query_string = f"status=200&contenttype={content_type}&response={response}&nocache={random.random()}"
    return f"/webdriver/tests/support/http_handlers/cached.py?{query_string}"


# Array of status and status text expected to be available in network events
HTTP_STATUS_AND_STATUS_TEXT = [
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
]

BASE_URL = "/webdriver/tests/bidi/network/support"
PAGE_DATA_URL_HTML = "data:text/html,<div>foo</div>"
PAGE_DATA_URL_IMAGE = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEX/TQBcNTh/AAAAAXRSTlPM0jRW/QAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII="
PAGE_EMPTY_HTML = f"{BASE_URL}/empty.html"
PAGE_EMPTY_IMAGE = f"{BASE_URL}/empty.png"
PAGE_EMPTY_SCRIPT = f"{BASE_URL}/empty.js"
PAGE_EMPTY_SVG = f"{BASE_URL}/empty.svg"
PAGE_EMPTY_TEXT = f"{BASE_URL}/empty.txt"
PAGE_INVALID_URL = "https://not_a_valid_url.test/"
PAGE_INITIATOR = {
    "HTML": f"{BASE_URL}/initiator/simple-initiator.html",
    "SCRIPT": f"{BASE_URL}/initiator/simple-initiator-script.js",
    "STYLESHEET": f"{BASE_URL}/initiator/simple-initiator-style.css",
    "IMAGE": f"{BASE_URL}/initiator/simple-initiator-img.png",
    "BACKGROUND": f"{BASE_URL}/initiator/simple-initiator-bg.png",
}
PAGE_OTHER_TEXT = f"{BASE_URL}/other.txt"
PAGE_PROVIDE_RESPONSE_HTML = f"{BASE_URL}/provide_response.html"
PAGE_PROVIDE_RESPONSE_SCRIPT = f"{BASE_URL}/provide_response.js"
PAGE_PROVIDE_RESPONSE_STYLESHEET = f"{BASE_URL}/provide_response.css"
PAGE_REDIRECT_HTTP_EQUIV = (
    "/webdriver/tests/bidi/network/support/redirect_http_equiv.html"
)
PAGE_REDIRECTED_HTML = "/webdriver/tests/bidi/network/support/redirected.html"
PAGE_SERVICEWORKER_HTML = "/webdriver/tests/bidi/network/support/serviceworker.html"

IMAGE_RESPONSE_BODY = urllib.parse.quote_plus(base64.b64decode(b"iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg=="))

SCRIPT_CONSOLE_LOG = urllib.parse.quote_plus("console.log('test')")
SCRIPT_CONSOLE_LOG_IN_MODULE = urllib.parse.quote_plus("export default function foo() { console.log('from module') }")

STYLESHEET_GREY_BACKGROUND = urllib.parse.quote_plus("html, body { background-color: #ccc; }")
STYLESHEET_RED_COLOR = urllib.parse.quote_plus("html, body { color: red; }")

AUTH_REQUIRED_EVENT = "network.authRequired"
BEFORE_REQUEST_SENT_EVENT = "network.beforeRequestSent"
FETCH_ERROR_EVENT = "network.fetchError"
RESPONSE_COMPLETED_EVENT = "network.responseCompleted"
RESPONSE_STARTED_EVENT = "network.responseStarted"

PHASE_TO_EVENT_MAP = {
    "authRequired": [AUTH_REQUIRED_EVENT, assert_response_event],
    "beforeRequestSent": [BEFORE_REQUEST_SENT_EVENT, assert_before_request_sent_event],
    "responseStarted": [RESPONSE_STARTED_EVENT, assert_response_event],
}

expires_a_day_from_now = datetime.now(timezone.utc) + timedelta(days=1)
expires_a_day_from_now_timestamp = int(expires_a_day_from_now.timestamp())
# Bug 1916221, the parsed expiry can have a slightly different value than the
# computed timestamp as Firefox tries to accommodate for the difference between
# the server clock and the system clock.
expires_interval = int_interval(
    expires_a_day_from_now_timestamp - 1,
    expires_a_day_from_now_timestamp + 1,
)

# Common parameters for Set-Cookie headers tests used for network interception
# commands.
#
# Note that the domain needs to be handled separately because the actual
# value will be retrieved via the domain_value fixture.
# with_domain can either be :
#  - "default": domain will be set to domain_value() and the page will be
#    loaded on domain_value().
#  - "alt": domain will be set to domain_value(alt) and the page will be
#    loaded on domain_value(alt).
#  - None (or any other value): domain will not be set and the page will be
#    loaded on domain_value() (which is the default).
SET_COOKIE_TEST_PARAMETERS = [
    (
        SetCookieHeader(
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        "default domain",
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        "alt domain",
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            name="foo",
            path="/some/other/path",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/some/other/path",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            http_only=True,
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": True,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            name="foo",
            path="/",
            secure=True,
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": True,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            expiry=expires_a_day_from_now.strftime("%a, %d %b %Y %H:%M:%S"),
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "expiry": expires_interval,
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            max_age=3600,
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "expiry": any_int,
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            same_site="none",
            # SameSite None requires Secure to set the cookie correctly.
            secure=True,
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "none",
            "secure": True,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            same_site="lax",
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "lax",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
    (
        SetCookieHeader(
            same_site="strict",
            name="foo",
            path="/",
            value=NetworkStringValue("bar"),
        ),
        None,
        {
            "httpOnly": False,
            "name": "foo",
            "path": "/",
            "sameSite": "strict",
            "secure": False,
            "size": 6,
            "value": {"type": "string", "value": "bar"},
        },
    ),
]

SET_COOKIE_TEST_IDS=[
    "no domain",
    "default domain",
    "alt domain",
    "custom path",
    "http only",
    "secure",
    "expiry",
    "max age",
    "same site none",
    "same site lax",
    "same site strict",
]
