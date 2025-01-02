import pytest

from webdriver.bidi.modules.network import NetworkStringValue

from ... import recursive_compare
from .. import assert_response_event, RESPONSE_COMPLETED_EVENT

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
