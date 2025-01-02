import pytest

from webdriver.bidi.modules.network import (
    CookieHeader,
    Header,
    NetworkStringValue,
    SetCookieHeader,
)

from .. import (
    assert_response_event,
    PAGE_EMPTY_TEXT,
    PAGE_PROVIDE_RESPONSE_HTML,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
    SET_COOKIE_TEST_IDS,
    SET_COOKIE_TEST_PARAMETERS
)

from ... import any_int, recursive_compare

pytestmark = pytest.mark.asyncio

LOAD_EVENT = "browsingContext.load"


async def test_cookie_response_started(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    fetch,
    url,
):
    request = await setup_blocked_request(
        phase="responseStarted",
        navigate=True,
        blocked_url=url(PAGE_PROVIDE_RESPONSE_HTML),
        navigate_url=url(PAGE_PROVIDE_RESPONSE_HTML),
    )

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            LOAD_EVENT,
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    on_load = wait_for_event(LOAD_EVENT)

    # Prepare the cookie and header values to set and assert a test cookie
    cookie_name = "test-cookie"
    cookie_value = "test-cookie-value"
    request_cookie = CookieHeader(
        name=cookie_name, value=NetworkStringValue(cookie_value)
    )
    response_cookie = SetCookieHeader(
        name=cookie_name, value=NetworkStringValue(cookie_value), path="/"
    )
    set_cookie_header = Header(
        name="Set-Cookie",
        value=NetworkStringValue("test-cookie=test-cookie-value;Path=/"),
    )

    await bidi_session.network.continue_response(
        request=request,
        cookies=[response_cookie],
    )

    # Check that the response events contain the expected Set-Cookie header.
    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event, expected_response={"headers": [set_cookie_header]}
    )

    # Wait for the navigation to complete.
    await wait_for_future_safe(on_load)

    # Perform a fetch from the page.
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(PAGE_EMPTY_TEXT)
    response_completed_event = await wait_for_future_safe(on_response_completed)

    # Check that the fetch contains the cookie set with provideResponse.
    assert_response_event(
        response_completed_event, expected_request={"cookies": [request_cookie]}
    )

    await bidi_session.storage.delete_cookies()


@pytest.mark.parametrize(
    "cookie, with_domain, expected_cookie",
    SET_COOKIE_TEST_PARAMETERS,
    ids=SET_COOKIE_TEST_IDS,
)
async def test_cookie_attributes_response_started(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
    domain_value,
    cookie,
    with_domain,
    expected_cookie,
):
    if with_domain == "default domain":
        domain = ""
        cookie["domain"] = domain_value()
        expected_cookie["domain"] = f".{domain_value()}"
    elif with_domain == "alt domain":
        domain = "alt"
        cookie["domain"] = domain_value("alt")
        expected_cookie["domain"] = f".{domain_value('alt')}"
    else:
        # If the cookie is not set for a specific domain it will default to
        # the current domain, but no "." will be prepended to the actual cookie
        # domain
        domain = ""
        expected_cookie["domain"] = domain_value()

    request = await setup_blocked_request(
        phase="responseStarted",
        navigate=True,
        blocked_url=url(PAGE_PROVIDE_RESPONSE_HTML, domain=domain),
    )

    await subscribe_events(events=[LOAD_EVENT])

    on_load = wait_for_event(LOAD_EVENT)

    # Provide response with an empty cookies list
    await bidi_session.network.continue_response(
        request=request,
        cookies=[cookie],
    )

    # Wait for the navigation to complete.
    await wait_for_future_safe(on_load)

    cookies = await bidi_session.storage.get_cookies()
    assert len(cookies["cookies"]) == 1

    cookie = cookies["cookies"][0]
    recursive_compare(expected_cookie, cookie)

    await bidi_session.storage.delete_cookies()


async def test_no_cookie_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    url,
):
    request = await setup_blocked_request(
        phase="responseStarted",
        navigate=True,
        blocked_url=url(PAGE_PROVIDE_RESPONSE_HTML),
        navigate_url=url(PAGE_PROVIDE_RESPONSE_HTML),
    )

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            LOAD_EVENT,
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    on_load = wait_for_event(LOAD_EVENT)

    # Provide response with an empty cookies list
    await bidi_session.network.continue_response(
        request=request,
        cookies=[],
    )

    # Check that the response events contain no Set-Cookie header.
    async def wait_for_event_and_assert_no_cookie(on_response_event):
        response_event = await wait_for_future_safe(on_response_event)
        response_headers = response_event["response"]["headers"]
        assert len([h for h in response_headers if h["name"] == "Set-Cookie"]) == 0

    await wait_for_event_and_assert_no_cookie(on_response_completed)

    # Wait for the navigation to complete.
    await wait_for_future_safe(on_load)

    await bidi_session.storage.delete_cookies()
