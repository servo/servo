#  TODO(#42482): Merge this file with response_completed.py
#
# The status codes in this file are currently problematic in some implementations.
#
# The only mechanism currently provided by WPT to disable subtests with
# expectations is to disable the entire file. As such, this file is a copy of
# response_completed.py with the problematic status codes extracted.
#
# Once it is possible to disable subtests, this file should be merged with
# response_completed.py.

import pytest

from .. import assert_response_event, HTTP_STATUS_AND_STATUS_TEXT

RESPONSE_COMPLETED_EVENT = "network.responseCompleted"


@pytest.mark.parametrize(
    "status, status_text",
    [(status, text) for (status, text) in HTTP_STATUS_AND_STATUS_TEXT if status in [101, 407]],
)
@pytest.mark.asyncio
async def test_response_status(
    wait_for_event, url, fetch, setup_network_test, status, status_text
):
    status_url = url(
        f"/webdriver/tests/support/http_handlers/status.py?status={status}&nocache={RESPONSE_COMPLETED_EVENT}"
    )

    network_events = await setup_network_test(events=[RESPONSE_COMPLETED_EVENT])
    events = network_events[RESPONSE_COMPLETED_EVENT]

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(status_url)
    await on_response_completed

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
