import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT

pytestmark = pytest.mark.asyncio


async def test_request(
    bidi_session,
    url,
    setup_collected_response,
):
    [request, collector] = await setup_collected_response(
        fetch_url=url(PAGE_EMPTY_TEXT)
    )
    await bidi_session.network.disown_data(
        request=request, data_type="response", collector=collector
    )

    # Check that after calling disown data, you can no longer get or disown the
    # data.
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.disown_data(
            request=request, data_type="response", collector=collector
        )

    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="response")
