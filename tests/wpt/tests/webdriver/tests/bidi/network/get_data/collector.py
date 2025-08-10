import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT

pytestmark = pytest.mark.asyncio


async def test_single_collector(bidi_session, url, setup_collected_response):
    [request, collector] = await setup_collected_response(
        fetch_url=url(PAGE_EMPTY_TEXT)
    )
    data = await bidi_session.network.get_data(
        request=request, data_type="response", collector=collector
    )

    assert data["type"] == "string"
    assert data["value"] == "empty\n"


async def test_several_collectors(
    bidi_session, url, add_data_collector, setup_collected_response
):
    collector = await add_data_collector(
        collector_type="blob", data_types=["response"], max_encoded_data_size=1000
    )
    [request, other_collector] = await setup_collected_response(
        fetch_url=url(PAGE_EMPTY_TEXT)
    )

    data = await bidi_session.network.get_data(
        request=request, data_type="response", collector=collector
    )
    assert data["type"] == "string"
    assert data["value"] == "empty\n"

    data_from_other_collector = await bidi_session.network.get_data(
        request=request, data_type="response", collector=other_collector
    )
    assert data_from_other_collector == data
