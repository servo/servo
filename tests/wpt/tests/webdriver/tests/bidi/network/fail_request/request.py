import pytest

from .. import (
    assert_fetch_error_event,
    PAGE_EMPTY_TEXT,
    FETCH_ERROR_EVENT,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("phase", ["beforeRequestSent", "responseStarted"])
async def test_phases(
    setup_blocked_request, subscribe_events, wait_for_event, bidi_session, url, phase
):
    request = await setup_blocked_request(phase)
    await subscribe_events(events=[FETCH_ERROR_EVENT])

    on_fetch_error = wait_for_event(FETCH_ERROR_EVENT)
    await bidi_session.network.fail_request(request=request)
    await on_fetch_error

    fetch_error_event = await on_fetch_error
    expected_request = {"method": "GET", "url": url(PAGE_EMPTY_TEXT)}
    assert_fetch_error_event(
        fetch_error_event,
        expected_request=expected_request,
        redirect_count=0,
    )
