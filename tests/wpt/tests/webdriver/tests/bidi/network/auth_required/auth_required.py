import asyncio
import pytest
from webdriver.bidi.modules.network import AuthCredentials
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll

from ... import number_interval
from .. import (
    assert_response_event,
    AUTH_REQUIRED_EVENT,
    PAGE_EMPTY_HTML,
)


@pytest.mark.asyncio
async def test_subscribe_status(
    bidi_session, new_tab, subscribe_events, wait_for_event, wait_for_future_safe, url, fetch
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    await subscribe_events(events=[AUTH_REQUIRED_EVENT])

    # Track all received network.authRequired events in the events array.
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        AUTH_REQUIRED_EVENT, on_event)

    auth_url = url(
        "/webdriver/tests/support/http_handlers/authentication.py?realm=testrealm"
    )

    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)

    asyncio.ensure_future(fetch(url=auth_url, context=new_tab))

    await wait_for_future_safe(on_auth_required)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": auth_url}
    expected_response = {
        "url": auth_url,
        "authChallenges": [
            ({"scheme": "Basic", "realm": "testrealm"}),
        ],
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        redirect_count=0,
    )

    remove_listener()


@pytest.mark.asyncio
async def test_no_authentication(
    bidi_session, new_tab, subscribe_events, url
):
    await subscribe_events(events=[AUTH_REQUIRED_EVENT])

    # Track all received network.authRequired events in the events array.
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        AUTH_REQUIRED_EVENT, on_event)

    # Navigate to a page which should not trigger any authentication.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url(PAGE_EMPTY_HTML),
        wait="complete",
    )

    assert len(events) == 0
    remove_listener()


@pytest.mark.asyncio
async def test_request_timing_info(
    bidi_session,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
    current_time,
):
    network_events = await setup_network_test(
        events=[AUTH_REQUIRED_EVENT], context=new_tab["context"]
    )
    events = network_events[AUTH_REQUIRED_EVENT]

    # Record the time range for the request to assert the timing info.
    time_start = await current_time()

    auth_url = url(
        "/webdriver/tests/support/http_handlers/authentication.py?realm=testrealm"
    )

    on_auth_required = wait_for_event(AUTH_REQUIRED_EVENT)
    asyncio.ensure_future(fetch(url=auth_url, context=new_tab))
    await wait_for_future_safe(on_auth_required)

    time_end = await current_time()
    time_range = number_interval(time_start, time_end)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": auth_url}
    expected_response = {
        "url": auth_url,
        "authChallenges": [
            ({"scheme": "Basic", "realm": "testrealm"}),
        ],
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
        expected_time_range=time_range,
        redirect_count=0,
    )


@pytest.mark.asyncio
async def test_with_wrong_credentials(setup_blocked_request, bidi_session):
    # Setup unique username / password because browsers cache credentials.
    username = "test_with_wrong_credentials"
    password = "test_with_wrong_credentials_password"
    request = await setup_blocked_request(
        "authRequired", username=username, password=password
    )

    # Track all received network.authRequired events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(AUTH_REQUIRED_EVENT, on_event)
    wait = AsyncPoll(bidi_session, timeout=1)

    wrong_credentials = AuthCredentials(username=username, password="wrong_password")
    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=wrong_credentials
    )

    # We expect to get authRequired event after providing wrong credentials
    await wait.until(lambda _: len(events) > 0)

    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=wrong_credentials
    )

    # We expect to get another authRequired event after providing wrong credentials
    await wait.until(lambda _: len(events) > 1)

    # Check no other authRequired event was received
    wait = AsyncPoll(bidi_session, timeout=1)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 2)

    remove_listener()
