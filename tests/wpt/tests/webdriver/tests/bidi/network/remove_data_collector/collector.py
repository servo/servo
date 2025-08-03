import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


async def test_return_value(bidi_session, add_intercept):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    result = await bidi_session.network.remove_data_collector(collector=collector)
    assert result == {}
