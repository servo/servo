import pytest

from .. import RESPONSE_COMPLETED_EVENT, RESPONSE_STARTED_EVENT

pytestmark = pytest.mark.asyncio


async def test_continue_fetch_request(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session
):
    request = await setup_blocked_request("beforeRequestSent")

    await subscribe_events(
        events=[
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
        ]
    )

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    await bidi_session.network.continue_request(request=request)

    await on_response_started
    await on_response_completed


async def test_continue_navigation(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session
):
    request = await setup_blocked_request("beforeRequestSent", navigate=True)

    await subscribe_events(
        events=[
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
            "browsingContext.load",
        ]
    )

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    on_load = wait_for_event("browsingContext.load")

    await bidi_session.network.continue_request(request=request)

    await on_response_started
    await on_response_completed
    await on_load
