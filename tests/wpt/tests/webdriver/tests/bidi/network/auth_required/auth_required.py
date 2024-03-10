import pytest

import asyncio

from .. import assert_response_event, AUTH_REQUIRED_EVENT, PAGE_EMPTY_HTML


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
