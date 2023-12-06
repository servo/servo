# META: timeout=long

import pytest

from .. import (
    assert_before_request_sent_event,
    assert_response_event,
)

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"
PAGE_OTHER_TEXT = "/webdriver/tests/bidi/network/support/other.txt"


@pytest.mark.asyncio
@pytest.mark.parametrize("phase", [
    "beforeRequestSent",
    "responseStarted",
])
async def test_remove_intercept(
    bidi_session, wait_for_event, url, setup_network_test, add_intercept, subscribe_events, top_context, wait_for_future_safe, phase
):
    network_events = await setup_network_test(
        events=[
            "network.beforeRequestSent",
            "network.responseStarted",
            "network.responseCompleted",
        ]
    )
    before_request_sent_events = network_events["network.beforeRequestSent"]
    response_started_events = network_events["network.responseStarted"]
    response_completed_events = network_events["network.responseCompleted"]

    text_url = url(PAGE_EMPTY_TEXT)
    intercept = await add_intercept(
        phases=[phase],
        url_patterns=[{"type": "string", "pattern": text_url}],
    )

    on_network_event = wait_for_event(f"network.{phase}")

    await subscribe_events(events=["browsingContext.load"], contexts=[top_context["context"]])

    browsing_context_load_events = []

    async def on_browsing_context_load_event(method, data):
        browsing_context_load_events.append(data)

    remove_listener = bidi_session.add_event_listener("browsingContext.load", on_browsing_context_load_event)

    # Request to top_context should be blocked.
    # TODO(https://github.com/w3c/webdriver-bidi/issues/188): Use a timeout argument when available.
    await bidi_session.browsing_context.navigate(context=top_context["context"], url=text_url, wait="complete")

    await wait_for_future_safe(on_network_event)

    assert len(before_request_sent_events) == 1

    if phase == "beforeRequestSent":
        assert len(response_started_events) == 0
        assert_before_request_sent_event(
            before_request_sent_events[0], is_blocked=True, intercepts=[intercept]
        )
    elif phase == "responseStarted":
        assert len(response_started_events) == 1
        assert_before_request_sent_event(
            before_request_sent_events[0], is_blocked=False
        )
        assert_response_event(
            response_started_events[0], is_blocked=True, intercepts=[intercept]
        )

    # Check that we did not receive response completed events.
    assert len(response_completed_events) == 0

    # Remove the intercept
    await bidi_session.network.remove_intercept(intercept=intercept)

    # The next request should not be blocked
    on_response_completed = wait_for_event("network.responseCompleted")
    await bidi_session.browsing_context.navigate(context=top_context["context"], url=text_url, wait="complete")
    await wait_for_future_safe(on_response_completed)

    # Assert the network events have the expected interception properties
    assert len(before_request_sent_events) == 2
    assert_before_request_sent_event(before_request_sent_events[1], is_blocked=False)

    if phase == "beforeRequestSent":
        assert len(response_started_events) == 1
        assert_response_event(response_started_events[0], is_blocked=False)
    elif phase == "responseStarted":
        assert len(response_started_events) == 2
        assert_response_event(response_started_events[1], is_blocked=False)

    assert len(response_completed_events) == 1
    assert_response_event(response_completed_events[0], is_blocked=False)

    assert len(browsing_context_load_events) == 0
    remove_listener()


@pytest.mark.asyncio
async def test_return_value(bidi_session, add_intercept):
    intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[],
    )

    result = await bidi_session.network.remove_intercept(intercept=intercept)
    assert result == {}
