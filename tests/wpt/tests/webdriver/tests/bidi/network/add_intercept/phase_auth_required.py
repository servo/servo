import pytest

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
)

PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"

AUTH_REQUIRED_EVENT = "network.authRequired"

pytestmark = pytest.mark.asyncio


async def test_basic_authentication(
    bidi_session,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    url,
    setup_network_test,
    add_intercept,
):
    network_events = await setup_network_test(
        events=[
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.authRequired",
            "network.responseCompleted",
        ]
    )
    before_request_sent_events = network_events["network.beforeRequestSent"]
    response_started_events = network_events["network.responseStarted"]
    auth_required_events = network_events["network.authRequired"]
    response_completed_events = network_events["network.responseCompleted"]

    auth_url = url("/webdriver/tests/support/http_handlers/authentication.py")
    intercept = await add_intercept(
        phases=["authRequired"],
        url_patterns=[{"type": "string", "pattern": auth_url}],
    )

    assert isinstance(intercept, str)

    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=auth_url,
        wait="none",
    )

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
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.authRequired",
            "network.responseCompleted",
        ]
    )
    before_request_sent_events = network_events["network.beforeRequestSent"]
    response_started_events = network_events["network.responseStarted"]
    auth_required_events = network_events["network.authRequired"]
    response_completed_events = network_events["network.responseCompleted"]

    text_url = url(PAGE_EMPTY_TEXT)
    intercept = await add_intercept(
        phases=["authRequired"],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    assert isinstance(intercept, str)

    on_network_event = wait_for_event("network.responseCompleted")

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
