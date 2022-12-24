from .. import any_int, any_list, any_string, recursive_compare


def assert_timing_info(timing_info):
    recursive_compare(
        {
            "requestTime": any_int,
            "redirectStart": any_int,
            "redirectEnd": any_int,
            "fetchStart": any_int,
            "dnsStart": any_int,
            "dnsEnd": any_int,
            "connectStart": any_int,
            "connectEnd": any_int,
            "tlsStart": any_int,
            "tlsEnd": any_int,
            "requestStart": any_int,
            "responseStart": any_int,
            "responseEnd": any_int,
        },
        timing_info,
    )


def assert_request_data(request_data, cookies, headers, method, url):
    recursive_compare(
        {
            "bodySize": any_int,
            "cookies": any_list,
            "headers": any_list,
            "headersSize": any_int,
            "method": any_string,
            "request": any_string,
            "url": any_string,
        },
        request_data,
    )

    if cookies is not None:
        request_cookies = request_data["cookies"]
        assert len(request_cookies) == len(cookies)

        # Simple helper to find a cookie by key and value only.
        def match_cookie(cookie, expected):
            for key in expected:
                if cookie[key] != expected[key]:
                    return False

            return True

        for cookie in cookies:
            assert next(c for c in request_cookies if match_cookie(c, cookie)) is not None

    if headers is not None:
        request_headers = request_data["headers"]
        # The browser sets request headers, only assert that the expected headers
        # are included in the request's headers.
        assert len(request_headers) >= len(headers)
        for header in headers:
            assert next(h for h in request_headers if header == h) is not None

    if method is not None:
        assert request_data["method"] == method

    if url is not None:
        assert request_data["url"] == url

    assert_timing_info(request_data["timings"])


def assert_base_parameters(
    event,
    context=None,
    cookies=None,
    headers=None,
    is_redirect=None,
    method=None,
    redirect_count=None,
    url=None,
):
    # Assert context
    assert isinstance(event["context"], str)
    if context is not None:
        assert event["context"] == context

    # Assert isRedirect
    assert isinstance(event["isRedirect"], bool)
    if is_redirect is not None:
        assert event["isRedirect"] == is_redirect

    # Assert redirectCount
    assert isinstance(event["redirectCount"], int)
    if redirect_count is not None:
        assert event["redirectCount"] == redirect_count

    # Assert navigation
    assert "navigation" in event

    # Assert request data
    assert "request" in event
    assert_request_data(
        event["request"],
        cookies=cookies,
        headers=headers,
        method=method,
        url=url,
    )

    # Assert BaseParameters' timestamp
    assert isinstance(event["timestamp"], int)


def assert_before_request_sent_event(
    event,
    context=None,
    cookies=None,
    headers=None,
    is_redirect=None,
    method=None,
    redirect_count=None,
    url=None,
):
    # Assert initiator
    assert isinstance(event["initiator"], dict)
    assert isinstance(event["initiator"]["type"], str)

    # Assert base parameters
    assert_base_parameters(
        event,
        context=context,
        cookies=cookies,
        headers=headers,
        is_redirect=is_redirect,
        method=method,
        redirect_count=redirect_count,
        url=url,
    )
