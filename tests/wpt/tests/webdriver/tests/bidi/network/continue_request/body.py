import asyncio
import json

import pytest

from webdriver.bidi.modules.network import NetworkStringValue

from .. import BEFORE_REQUEST_SENT_EVENT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "request_post_data, modified_post_data, expected_size",
    [
        ["{'a': 1}", "", 0],
        [None, "{'a': 123}", 10],
        ["{'a': 1}", "{'a': 12345678}", 15],
    ],
)
async def test_request_body(
    bidi_session,
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    request_post_data,
    modified_post_data,
    expected_size,
):
    request = await setup_blocked_request(
        "beforeRequestSent", method="POST", post_data=request_post_data
    )
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    body = NetworkStringValue(modified_post_data)
    await bidi_session.network.continue_request(request=request, body=body)
    response_event = await on_response_completed
    assert response_event["request"]["bodySize"] == expected_size


@pytest.mark.parametrize(
    "request_post_data, modified_post_data",
    [
        [None, "{'a': 123}"],
        ["{'a': 1}", "{'a': 12345678}"],
        ["{'a': 12345678}", "{'a': 1}"],
    ],
    ids=["no original body", "longer", "shorter"],
)
async def test_request_body_received_by_server(
    bidi_session,
    setup_network_test,
    add_intercept,
    fetch,
    wait_for_event,
    wait_for_future_safe,
    url,
    request_post_data,
    modified_post_data,
):
    echo_url = url("/webdriver/tests/support/http_handlers/body_echo.py")

    await setup_network_test(events=[BEFORE_REQUEST_SENT_EVENT])
    await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": echo_url}],
    )

    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    fetch_task = asyncio.ensure_future(
        fetch(echo_url, method="POST", post_data=request_post_data)
    )
    event = await wait_for_future_safe(on_before_request_sent)

    await bidi_session.network.continue_request(
        request=event["request"]["request"],
        body=NetworkStringValue(modified_post_data),
    )

    result = await fetch_task
    echo = json.loads(result["value"])
    assert echo == {
        "body": modified_post_data,
        "contentLength": str(len(modified_post_data)),
    }
