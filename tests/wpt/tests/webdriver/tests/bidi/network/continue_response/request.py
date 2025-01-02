import pytest

from .. import AUTH_REQUIRED_EVENT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_continue_auth_required(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    navigate,
    wait_for_future_safe,
):
    # Setup unique username / password because browsers cache credentials.
    username = f"test_continue_auth_required_{navigate}"
    password = f"test_continue_auth_required_password_{navigate}"
    request = await setup_blocked_request(
        "authRequired", username=username, password=password, navigate=navigate
    )

    await subscribe_events(
        events=[
            AUTH_REQUIRED_EVENT,
        ]
    )

    # Continue the request blocked on authRequired. Without credentials, another
    # network.authRequired should be emitted.
    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)
    await bidi_session.network.continue_response(request=request)
    await wait_for_future_safe(on_auth_required)


@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_continue_response_started(
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    bidi_session,
    navigate,
    wait_for_future_safe,
):
    request = await setup_blocked_request("responseStarted", navigate=navigate)

    await subscribe_events(
        events=[
            RESPONSE_COMPLETED_EVENT,
            "browsingContext.load",
        ]
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    if navigate:
        on_load = wait_for_event("browsingContext.load")

    await bidi_session.network.continue_response(request=request)

    await wait_for_future_safe(on_response_completed)
    if navigate:
        await wait_for_future_safe(on_load)
