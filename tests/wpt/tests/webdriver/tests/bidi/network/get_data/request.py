import asyncio
import pytest

from .. import (
    PAGE_EMPTY_TEXT,
    RESPONSE_STARTED_EVENT,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


async def test_redirect(bidi_session, url, setup_collected_data):
    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}"
    )

    [request, _] = await setup_collected_data(fetch_url=redirect_url)
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    assert data["value"] == "empty\n"


async def test_redirect_race_condition(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    new_tab,
    url,
    fetch,
    setup_network_test,
    add_data_collector,
):
    network_events = await setup_network_test(
        events=[RESPONSE_STARTED_EVENT, RESPONSE_COMPLETED_EVENT],
        context=new_tab["context"],
    )

    await add_data_collector(
        max_encoded_data_size=100000, data_types=["response"]
    )

    on_response_started = wait_for_event(RESPONSE_STARTED_EVENT)
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    text_url = url(PAGE_EMPTY_TEXT)
    redirect_url = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={text_url}"
    )

    fetch_task = asyncio.ensure_future(fetch(redirect_url, context=new_tab))

    # Call get_data immediately after receiving network.responseStarted.
    event = await wait_for_future_safe(on_response_started)
    data = await bidi_session.network.get_data(
        request=event["request"]["request"], data_type="response"
    )

    assert data["type"] == "string"
    assert data["value"] == "empty\n"

    await wait_for_future_safe(fetch_task)
    await wait_for_future_safe(on_response_completed)

    assert len(network_events[RESPONSE_STARTED_EVENT]) == 2
    assert len(network_events[RESPONSE_COMPLETED_EVENT]) == 2
