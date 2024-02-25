import pytest

from webdriver.bidi.modules.network import AuthCredentials

from tests.support.sync import AsyncPoll

from .. import AUTH_REQUIRED_EVENT, RESPONSE_COMPLETED_EVENT, RESPONSE_STARTED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_provide_response_auth_required(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, navigate
):
    request = await setup_blocked_request("authRequired", navigate=navigate)

    await subscribe_events(
        events=[
            AUTH_REQUIRED_EVENT,
            "browsingContext.load",
        ]
    )

    # For requests blocked on authRequired, providing a response with no
    # additional argument should just lead to another authRequired event.
    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)

    await bidi_session.network.provide_response(request=request)

    await on_auth_required


@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_provide_response_phase(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, phase, navigate
):
    request = await setup_blocked_request(phase, navigate=navigate)

    await subscribe_events(
        events=[
            RESPONSE_STARTED_EVENT,
            RESPONSE_COMPLETED_EVENT,
            "browsingContext.load",
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    if phase == "beforeRequestSent":
        # For a request blocked on beforeRequestSent, a responseStarted event is
        # also expected.
        on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)

    if navigate:
        on_load = wait_for_event("browsingContext.load")

    await bidi_session.network.provide_response(request=request)

    await on_response_completed

    if phase == "beforeRequestSent":
        await on_response_started

    if navigate:
        await on_load
