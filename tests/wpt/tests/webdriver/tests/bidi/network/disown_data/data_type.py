import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("data_type", ["request", "response"])
async def test_data_type(
    bidi_session,
    url,
    setup_collected_data,
    data_type,
):
    [request, collector] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        fetch_post_data="somedata",
        data_types=[data_type],
    )
    await bidi_session.network.disown_data(
        request=request, data_type=data_type, collector=collector
    )

    # Check that after calling disown data, you can no longer get or disown the
    # data.
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type=data_type, collector=collector
        )

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type=data_type)
