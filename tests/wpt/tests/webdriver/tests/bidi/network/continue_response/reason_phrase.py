import pytest

from webdriver.bidi.modules.network import NetworkStringValue

from .. import (
    assert_response_event,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


async def test_reason_phrase(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
):
    request = await setup_blocked_request(phase="responseStarted")

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    reason_phrase = "OK from continueResponse"
    await bidi_session.network.continue_response(
        request=request,
        reason_phrase=reason_phrase,
    )

    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event,
        expected_response={"statusText": reason_phrase},
    )


async def test_reason_phrase_and_status_code(
    setup_blocked_request,
    subscribe_events,
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
):
    request = await setup_blocked_request(phase="responseStarted")

    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    # Modify both the reason phrase and the status code in the same
    # continueResponse command.
    reason_phrase = "custom status text"
    status_code = 404

    await bidi_session.network.continue_response(
        request=request,
        reason_phrase=reason_phrase,
        status_code=status_code,
    )

    response_completed_event = await wait_for_future_safe(on_response_completed)
    assert_response_event(
        response_completed_event,
        expected_response={"statusText": reason_phrase, "status": status_code},
    )
