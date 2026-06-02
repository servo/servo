import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


async def test_return_value(bidi_session, add_data_collector):
    collector = await add_data_collector(
        collector_type="blob", data_types=["response"], max_encoded_data_size=1000
    )
    other_collector = await add_data_collector(
        collector_type="blob", data_types=["response"], max_encoded_data_size=1000
    )
    assert isinstance(collector, str)
    assert isinstance(other_collector, str)
    assert collector != other_collector
