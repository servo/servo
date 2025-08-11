import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_IMAGE, PAGE_EMPTY_TEXT, PAGE_OTHER_TEXT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "file, expected_value", [(PAGE_EMPTY_TEXT, "empty\n"), (PAGE_OTHER_TEXT, "other\n")]
)
async def test_request_text_file(
    bidi_session, url, setup_collected_response, file, expected_value
):
    [request, _] = await setup_collected_response(fetch_url=url(file))
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    assert data["value"] == expected_value


async def test_request_base64_file(
    bidi_session, url, setup_collected_response,
):
    [request, _] = await setup_collected_response(fetch_url=url(PAGE_EMPTY_IMAGE))
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "base64"
    assert isinstance(data["value"], str)
