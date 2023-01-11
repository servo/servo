import asyncio

import pytest

from .. import assert_before_request_sent_event, assert_response_started_event

PAGE_EMPTY_HTML = "/webdriver/tests/bidi/network/support/empty.html"
PAGE_EMPTY_TEXT = "/webdriver/tests/bidi/network/support/empty.txt"

# The following tests are marked as tentative until
# https://github.com/w3c/webdriver-bidi/pull/204 is merged.


@pytest.mark.asyncio
async def test_same_request_id(
    bidi_session, top_context, wait_for_event, url, setup_network_test, fetch
):
    network_events = await setup_network_test(
        events=["network.beforeRequestSent", "network.responseStarted"]
    )
    before_request_sent_events = network_events["network.beforeRequestSent"]
    response_started_events = network_events["network.responseStarted"]

    text_url = url(PAGE_EMPTY_TEXT)
    on_response_started = wait_for_event("network.responseStarted")
    await fetch(text_url)
    await on_response_started

    assert len(before_request_sent_events) == 1
    assert len(response_started_events) == 1
    expected_request = {"method": "GET", "url": text_url}
    assert_before_request_sent_event(
        before_request_sent_events[0], expected_request=expected_request
    )

    expected_response = {"url": text_url}
    assert_response_started_event(
        response_started_events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    assert (
        before_request_sent_events[0]["request"]["request"]
        == response_started_events[0]["request"]["request"]
    )
