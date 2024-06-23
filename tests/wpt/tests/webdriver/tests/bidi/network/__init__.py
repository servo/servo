from .. import (
    any_bool,
    any_dict,
    any_int,
    any_int_or_null,
    any_list,
    any_string,
    any_string_or_null,
    assert_cookies,
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


def assert_timing_info(timing_info):
    recursive_compare(
        {
            "timeOrigin": any_int,
            "requestTime": any_int,
            "redirectStart": any_int,
            "redirectEnd": any_int,
            "fetchStart": any_int,
            "dnsStart": any_int,
            "dnsEnd": any_int,
            "connectStart": any_int,
            "connectEnd": any_int,
            "tlsStart": any_int,
            "requestStart": any_int,
            "responseStart": any_int,
            "responseEnd": any_int,
        },
        timing_info,
    )


def assert_request_data(request_data, expected_request):
    recursive_compare(
        {
            "bodySize": any_int_or_null,
            "cookies": any_list,
            "headers": any_list,
            "headersSize": any_int,
            "method": any_string,
            "request": any_string,
            "timings": any_dict,
            "url": any_string,
        },
        request_data,
    )

    assert_timing_info(request_data["timings"])

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

    recursive_compare(expected_request, request_data)


def assert_base_parameters(
    event,
    context=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
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

    # Assert request data
    if expected_request is not None:
        assert_request_data(event["request"], expected_request)


def assert_before_request_sent_event(
    event,
    context=None,
    intercepts=None,
    is_blocked=None,
    navigation=None,
    redirect_count=None,
    expected_request=None,
):
    # Assert initiator
    assert isinstance(event["initiator"], dict)
    assert isinstance(event["initiator"]["type"], str)

    # Assert base parameters
    assert_base_parameters(
        event,
        context=context,
        intercepts=intercepts,
        is_blocked=is_blocked,
        navigation=navigation,
        redirect_count=redirect_count,
        expected_request=expected_request,
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

PAGE_DATA_URL_HTML = "data:text/html,<div>foo</div>"
PAGE_DATA_URL_IMAGE = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEX/TQBcNTh/AAAAAXRSTlPM0jRW/QAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII="
PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_IMAGE = "/webdriver/tests/bidi/network/support/empty.png"
PAGE_EMPTY_SCRIPT = "/webdriver/tests/bidi/network/support/empty.js"
PAGE_EMPTY_SVG = "/webdriver/tests/bidi/network/support/empty.svg"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"
PAGE_INVALID_URL = "https://not_a_valid_url.test/"
PAGE_OTHER_TEXT = "/webdriver/tests/bidi/network/support/other.txt"
PAGE_REDIRECT_HTTP_EQUIV = (
    "/webdriver/tests/bidi/network/support/redirect_http_equiv.html"
)
PAGE_REDIRECTED_HTML = "/webdriver/tests/bidi/network/support/redirected.html"
PAGE_SERVICEWORKER_HTML = "/webdriver/tests/bidi/network/support/serviceworker.html"

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
