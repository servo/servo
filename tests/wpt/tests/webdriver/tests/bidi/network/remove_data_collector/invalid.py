import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_collector_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.remove_data_collector(collector=value)


async def test_params_collector_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchNetworkCollectorException):
        await bidi_session.network.remove_data_collector(collector="does not exist")


async def test_params_collector_removed_collector(bidi_session):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    await bidi_session.network.remove_data_collector(collector=collector)

    with pytest.raises(error.NoSuchNetworkCollectorException):
        await bidi_session.network.remove_data_collector(collector=collector)
