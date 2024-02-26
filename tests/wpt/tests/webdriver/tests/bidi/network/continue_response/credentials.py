import pytest

from webdriver.bidi.modules.network import AuthCredentials

from tests.support.sync import AsyncPoll

from .. import AUTH_REQUIRED_EVENT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_wrong_credentials(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, navigate
):
    username = f"test_missing_credentials_{navigate}"
    password = f"test_missing_credentials_password_{navigate}"
    request = await setup_blocked_request(
        "authRequired", username=username, password=password, navigate=navigate
    )

    await subscribe_events(events=[AUTH_REQUIRED_EVENT])

    # Continue the request blocked on authRequired, with incorrect credentials.
    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)
    wrong_credentials = AuthCredentials(username=username, password="wrong_password")
    await bidi_session.network.continue_response(
        request=request, credentials=wrong_credentials
    )
    await on_auth_required


@pytest.mark.parametrize("navigate", [False, True], ids=["fetch", "navigate"])
async def test_correct_credentials(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, navigate
):
    # Setup unique username / password because browsers cache credentials.
    username = f"test_wrong_credentials_{navigate}"
    password = f"test_wrong_credentials_password_{navigate}"
    request = await setup_blocked_request(
        "authRequired", username=username, password=password, navigate=navigate
    )

    await subscribe_events(
        events=[AUTH_REQUIRED_EVENT, RESPONSE_COMPLETED_EVENT, "browsingContext.load"]
    )

    # Track all network.responseCompleted events.
    response_completed_events = []

    async def on_event(method, data):
        response_completed_events.append(data)

    remove_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )

    # Continue with the expected credentials.
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    if navigate:
        on_load = wait_for_event("browsingContext.load")

    correct_credentials = AuthCredentials(username=username, password=password)
    await bidi_session.network.continue_response(
        request=request, credentials=correct_credentials
    )
    await on_response_completed
    if navigate:
        await on_load

    # Wait until 2 responseCompleted events have been emitted:
    # - one for the initial request
    # - one for the continue with correct credentials
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(response_completed_events) >= 2)
    assert len(response_completed_events) == 2

    remove_listener()
