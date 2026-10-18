import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_disowned_collector(
    bidi_session,
    url,
    setup_collected_data,
):
    [request, collector] = await setup_collected_data(fetch_url=url(PAGE_EMPTY_TEXT))

    # disown using get_data
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=collector, disown=True
    )

    # Check that you can no longer disown data with
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type="response", collector=collector
        )


async def test_several_collectors(
    bidi_session,
    url,
    add_data_collector,
    setup_collected_data,
):
    collector = await add_data_collector(collector_type="blob", data_types=["response"])
    [request, other_collector] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT)
    )

    # disown with the first collector
    await bidi_session.network.disown_data(
        request=request, data_type="response", collector=collector
    )

    # Check that you can no longer get or disown data with the first collector,
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type="response", collector=collector
        )

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="response", collector=collector
        )

    # But the data should still be available from the other collector or without
    # a collector parameter.
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=other_collector
    )
    await bidi_session.network.get_data(request=request, data_type="response")

    # disown with the other collector
    await bidi_session.network.disown_data(
        request=request, data_type="response", collector=other_collector
    )

    # Check the data can no longer be retrieved or disowned
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type="response", collector=other_collector
        )

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="response")


async def test_disown_blocked_request(
    bidi_session,
    add_data_collector,
    setup_blocked_request,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    # Collect request body data and intercept the POST request before it is sent.
    collector = await add_data_collector(
        collector_type="blob", data_types=["request"]
    )
    request = await setup_blocked_request("beforeRequestSent", post_data="test")

    # Disowning collected data while blocked must not invalidate the request ID.
    await bidi_session.network.disown_data(
        request=request, data_type="request", collector=collector
    )

    # Check that the disowned data can no longer be retrieved while blocked.
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="request")

    # Verify the blocked request can still be resumed to completion.
    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await bidi_session.network.continue_request(request=request)
    await wait_for_future_safe(on_response_completed)

    # Check that the disowned data remains unavailable after completion.
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="request")
