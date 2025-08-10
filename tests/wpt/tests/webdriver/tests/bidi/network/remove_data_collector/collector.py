import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT

pytestmark = pytest.mark.asyncio


async def test_return_value(bidi_session):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    result = await bidi_session.network.remove_data_collector(collector=collector)
    assert result == {}


async def test_data_not_available_after_remove(
    bidi_session, url, add_data_collector, setup_collected_response
):
    # Collect a network response with 2 collectors
    collector = await add_data_collector(
        collector_type="blob", data_types=["response"], max_encoded_data_size=1000
    )
    [request, other_collector] = await setup_collected_response(
        fetch_url=url(PAGE_EMPTY_TEXT)
    )

    # Remove the first collector.
    await bidi_session.network.remove_data_collector(collector=collector)

    # Data still available from other collector and globally
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=other_collector
    )
    await bidi_session.network.get_data(request=request, data_type="response")

    # Remove the other collector.
    await bidi_session.network.remove_data_collector(collector=other_collector)

    # Data no longer available globally
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="response")
