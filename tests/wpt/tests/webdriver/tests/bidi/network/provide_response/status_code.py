import pytest

from webdriver.bidi.modules.network import NetworkStringValue

from .. import (
    assert_response_event,
    HTTP_STATUS_AND_STATUS_TEXT,
    RESPONSE_COMPLETED_EVENT,
    RESPONSE_STARTED_EVENT,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "status_code, status_text",
    HTTP_STATUS_AND_STATUS_TEXT,
)
async def test_status_code_before_request_sent(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    top_context,
    wait_for_event,
    wait_for_future_safe,
    status_code,
    status_text,
):
    request = await setup_blocked_request(phase="beforeRequestSent")

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            RESPONSE_STARTED_EVENT,
        ]
    )

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    await bidi_session.network.provide_response(
        request=request,
        body=NetworkStringValue("overridden response"),
        status_code=status_code,
        reason_phrase=status_text,
    )

    response_started_event = await wait_for_future_safe(on_response_started)
    assert_response_event(
        response_started_event,
        expected_response={"status": status_code, "statusText": status_text},
    )

    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event,
        expected_response={"status": status_code, "statusText": status_text},
    )
