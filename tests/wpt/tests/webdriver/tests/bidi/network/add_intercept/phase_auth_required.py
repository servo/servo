import pytest

import asyncio

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
    PAGE_EMPTY_TEXT,
    AUTH_REQUIRED_EVENT,
    BEFORE_REQUEST_SENT_EVENT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)

pytestmark = pytest.mark.asyncio


async def test_basic_authentication(
    bidi_session,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    url,
    setup_network_test,
    add_intercept,
    fetch,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_TEXT),
        wait="complete",
    )

    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            AUTH_REQUIRED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    before_request_sent_events = network_events[BEFORE_REQUEST_SENT_EVENT]
    response_started_events = network_events[RESPONSE_STARTED_EVENT]
    auth_required_events = network_events[AUTH_REQUIRED_EVENT]
    response_completed_events = network_events[RESPONSE_COMPLETED_EVENT]

    auth_url = url("/webdriver/tests/support/http_handlers/authentication.py")
    intercept = await add_intercept(
        phases=["authRequired"],
        url_patterns=[{"type": "string", "pattern": auth_url}],
    )

    assert isinstance(intercept, str)

    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)
    # The fetch should fails as there is no authentication
    asyncio.ensure_future(fetch(url=auth_url, context=new_tab))

    await wait_for_future_safe(on_auth_required)
    expected_request = {"method": "GET", "url": auth_url}

    assert len(before_request_sent_events) == 1
    assert len(response_started_events) == 1
    assert len(auth_required_events) == 1

    assert_before_request_sent_event(
        before_request_sent_events[0],
        expected_request=expected_request,
        is_blocked=False,
    )
    assert_response_event(
        response_started_events[0],
        expected_request=expected_request,
        is_blocked=False,
    )
    assert_response_event(
        auth_required_events[0],
        expected_request=expected_request,
        is_blocked=True,
        intercepts=[intercept],
    )

    # The request should remain blocked at the authRequired phase.
    assert len(response_completed_events) == 0


async def test_no_authentication(
    wait_for_event,
    url,
    setup_network_test,
    add_intercept,
    fetch,
    wait_for_future_safe,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
            RESPONSE_STARTED_EVENT,
            AUTH_REQUIRED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    before_request_sent_events = network_events[BEFORE_REQUEST_SENT_EVENT]
    response_started_events = network_events[RESPONSE_STARTED_EVENT]
    auth_required_events = network_events[AUTH_REQUIRED_EVENT]
    response_completed_events = network_events[RESPONSE_COMPLETED_EVENT]

    text_url = url(PAGE_EMPTY_TEXT)
    intercept = await add_intercept(
        phases=["authRequired"],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    assert isinstance(intercept, str)

    on_network_event = wait_for_event(RESPONSE_COMPLETED_EVENT)

    await fetch(text_url)
    await wait_for_future_safe(on_network_event)

    expected_request = {"method": "GET", "url": text_url}

    assert len(before_request_sent_events) == 1
    assert len(response_started_events) == 1
    assert len(response_completed_events) == 1

    # Check that no network event was blocked because of the authRequired
    # intercept since the URL does not trigger an auth prompt.
    assert_before_request_sent_event(
        before_request_sent_events[0],
        expected_request=expected_request,
        is_blocked=False,
    )
    assert_response_event(
        response_started_events[0],
        expected_request=expected_request,
        is_blocked=False,
    )
    assert_response_event(
        response_completed_events[0],
        expected_request=expected_request,
        is_blocked=False,
    )

    # No authRequired event should have been received.
    assert len(auth_required_events) == 0
