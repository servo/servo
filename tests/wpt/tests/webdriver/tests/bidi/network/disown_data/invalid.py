import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.disown_data(
            request=value, data_type="response", collector="collector_id"
        )


async def test_params_request_non_existent(bidi_session):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request="does_not_exist", data_type="response", collector=collector
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_data_type_invalid_type(bidi_session, value):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.disown_data(
            request="request_id", data_type=value, collector=collector
        )


@pytest.mark.parametrize("value", ["", "invalid"])
async def test_params_data_type_invalid_value(bidi_session, value):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.disown_data(
            request="request_id", data_type=value, collector=collector
        )


@pytest.mark.parametrize(
    "collector_data_type", ["request", "response"]
)
async def test_params_data_type_mismatch(
    bidi_session, url, setup_collected_data, collector_data_type
):
    [request, collector] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        data_types=[collector_data_type],
    )

    if collector_data_type == "request":
        data_type = "response"
    else:
        data_type = "request"

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type=data_type, collector=collector
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_collector_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.disown_data(
            request="request_id", data_type="response", collector=value
        )


async def test_params_collector_non_existent(bidi_session):
    with pytest.raises(error.NoSuchNetworkCollectorException):
        await bidi_session.network.disown_data(
            request="request_id", data_type="response", collector="does_not_exist"
        )


async def test_params_collector_removed_collector(bidi_session):
    collector = await bidi_session.network.add_data_collector(
        data_types=["response"], max_encoded_data_size=1000
    )

    await bidi_session.network.remove_data_collector(collector=collector)

    with pytest.raises(error.NoSuchNetworkCollectorException):
        await bidi_session.network.disown_data(
            request="request_id", data_type="response", collector=collector
        )


async def test_params_collector_not_in_collected_data(
    bidi_session, url, add_data_collector, setup_collected_data
):
    too_small_collector = await add_data_collector(
        data_types=["response"], max_encoded_data_size=1
    )
    [request, _] = await setup_collected_data(fetch_url=url(PAGE_EMPTY_TEXT))

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type="response", collector=too_small_collector
        )
