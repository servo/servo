import pytest

from webdriver.bidi.modules.network import NetworkStringValue

from .. import (
    assert_response_event,
    HTTP_STATUS_AND_STATUS_TEXT,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "status",
    HTTP_STATUS_AND_STATUS_TEXT,
)
async def test_status_code(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    status,
):
    request = await setup_blocked_request(phase="responseStarted")

    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    status_code = status[0]
    await bidi_session.network.continue_response(
        request=request,
        status_code=status_code,
    )

    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event,
        expected_response={"status": status_code},
    )
