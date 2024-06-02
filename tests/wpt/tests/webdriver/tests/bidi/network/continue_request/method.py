import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare
from .. import assert_response_event, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio

METHODS = [
    "DELETE",
    "GET",
    "HEAD",
    "OPTIONS",
    "PATCH",
    "POST",
    "PUT",
]


@pytest.mark.parametrize("request_method", METHODS)
@pytest.mark.parametrize("updated_method", METHODS)
async def test_request_method(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    request_method,
    updated_method,
):
    request = await setup_blocked_request("beforeRequestSent", method=request_method)
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request, method=updated_method)
    response_event = await on_response_completed
    assert_response_event(response_event, expected_request={"method": updated_method})
