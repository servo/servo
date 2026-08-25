import base64
import pytest
import webdriver.bidi.error as error

from tests.support.asserts import assert_png

from .. import (
    BEFORE_REQUEST_SENT_EVENT,
    PAGE_DATA_URL_IMAGE,
    PAGE_EMPTY_IMAGE,
    PAGE_EMPTY_TEXT,
    PAGE_OTHER_TEXT,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "file, expected_value", [(PAGE_EMPTY_TEXT, "empty\n"), (PAGE_OTHER_TEXT, "other\n")]
)
async def test_data_type_response_text_file(
    bidi_session, url, setup_collected_data, file, expected_value
):
    [request, _] = await setup_collected_data(fetch_url=url(file))
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    assert data["value"] == expected_value


async def test_data_type_response_base64_file(
    bidi_session,
    url,
    setup_collected_data,
):
    [request, _] = await setup_collected_data(fetch_url=url(PAGE_EMPTY_IMAGE))
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "base64"
    assert isinstance(data["value"], str)
    assert_png(data["value"])


async def test_data_type_response_data_scheme_text(
    bidi_session, url, setup_collected_data,
):
    expected_value = "loremipsum"
    [request, _] = await setup_collected_data(fetch_url=f"data:text/plain,{expected_value}")
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    assert data["value"] == expected_value


async def test_data_type_response_data_scheme_image(
    bidi_session, url, setup_collected_data,
):
    expected_value = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
    [request, _] = await setup_collected_data(fetch_url=f"data:image/png;base64,{expected_value}")
    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "base64"
    assert data["value"] == expected_value


async def test_data_type_response_empty_response(
    bidi_session,
    inline,
    setup_collected_data,
):
    empty_url = inline("", doctype="js")
    [request, _] = await setup_collected_data(fetch_url=empty_url)

    data = await bidi_session.network.get_data(request=request, data_type="response")

    assert data["type"] == "string"
    assert data["value"] == ""


async def test_data_type_request_text(bidi_session, url, setup_collected_data):
    [request, _] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        fetch_post_data="somedata",
        data_types=["request"],
    )
    data = await bidi_session.network.get_data(request=request, data_type="request")

    assert data["type"] == "string"
    assert data["value"] == "somedata"


async def test_data_type_request_text_multipart(bidi_session, url, setup_collected_data):
    [request, _] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        fetch_post_data={"foo": 1, "bar": 2},
        data_types=["request"],
    )

    data = await bidi_session.network.get_data(request=request, data_type="request")
    assert data["type"] == "string"
    assert 'form-data; name="foo"' in data["value"]
    assert 'form-data; name="bar"' in data["value"]


async def test_data_type_request_image(bidi_session, url, setup_collected_data):
    expected_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
    [request, _] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        fetch_post_data={
            "test_image": {
                "filename": "image.png",
                "type": "image/png",
                "value": expected_image,
            }
        },
        data_types=["request"],
    )

    data = await bidi_session.network.get_data(request=request, data_type="request")
    assert data["type"] == "base64"
    assert isinstance(data["value"], str)

    decoded_image = base64.b64decode(expected_image)
    decoded_value = base64.b64decode(data["value"])
    assert decoded_image in decoded_value


async def test_data_type_request_no_postdata(bidi_session, url, setup_collected_data):
    [request, _] = await setup_collected_data(
        fetch_url=url(PAGE_EMPTY_TEXT),
        data_types=["request"],
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="request")
