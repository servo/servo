import pytest
import webdriver.bidi.error as error
from webdriver.bidi.modules.network import AuthCredentials
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll
from .. import (
    assert_response_event,
    AUTH_REQUIRED_EVENT,
    PAGE_EMPTY_TEXT,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


async def test_cancel(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, url
):
    request = await setup_blocked_request("authRequired")
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_with_auth(request=request, action="cancel")
    await on_response_completed

    response_event = await on_response_completed
    assert_response_event(
        response_event,
        expected_response={
            "status": 401,
            "statusText": "Unauthorized",
        },
    )


async def test_default(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, url
):
    request = await setup_blocked_request("authRequired")

    # Additionally subscribe to all network events
    await subscribe_events(events=["network"])

    # Track all received network.responseCompleted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )

    # continueWithAuth using action "default" should show the authentication
    # prompt and no new network event should be generated.
    await bidi_session.network.continue_with_auth(request=request, action="default")

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


async def test_provideCredentials(
    setup_blocked_request, subscribe_events, bidi_session, url
):
    # Setup unique username / password because browsers cache credentials.
    username = "test_provideCredentials"
    password = "test_provideCredentials_password"
    request = await setup_blocked_request("authRequired", username=username, password=password)

    # Additionally subscribe to network.responseCompleted
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    # Track all received network.responseCompleted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )

    credentials = AuthCredentials(username=username, password=password)
    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=credentials
    )

    # TODO: At the moment, the specification does not expect to receive a
    # responseCompleted event for each authentication attempt, so only assert
    # the last event. See https://github.com/w3c/webdriver-bidi/issues/627

    # Wait until a a responseCompleted event with status 200 OK is received.
    wait = AsyncPoll(bidi_session, message="Didn't receive response completed events")
    await wait.until(lambda _: len(events) > 0 and events[-1]["response"]["status"] == 200)

    remove_listener()


async def test_provideCredentials_wrong_credentials(
    setup_blocked_request, subscribe_events, bidi_session, wait_for_event, url
):
    # Setup unique username / password because browsers cache credentials.
    username = "test_provideCredentials_wrong_credentials"
    password = "test_provideCredentials_wrong_credentials_password"
    request = await setup_blocked_request("authRequired", username=username, password=password)

    # Additionally subscribe to network.responseCompleted
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])

    # Track all received network.responseCompleted events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        RESPONSE_COMPLETED_EVENT, on_event
    )

    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)

    wrong_credentials = AuthCredentials(username=username, password="wrong_password")
    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=wrong_credentials
    )

    # We expect to get another authRequired event after providing wrong credentials
    await on_auth_required

    # Continue with the correct credentials
    correct_credentials = AuthCredentials(username=username, password=password)
    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=correct_credentials
    )

    # TODO: At the moment, the specification does not expect to receive a
    # responseCompleted event for each authentication attempt, so only assert
    # the last event. See https://github.com/w3c/webdriver-bidi/issues/627

    # Wait until a a responseCompleted event with status 200 OK is received.
    wait = AsyncPoll(bidi_session, message="Didn't receive response completed events")
    await wait.until(lambda _: len(events) > 0 and events[-1]["response"]["status"] == 200)

    remove_listener()
