import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


async def test_request_and_response_same_collector(
    bidi_session, url, setup_collected_data
):
    [request, _] = await setup_collected_data(
        fetch_url=url(
            "/webdriver/tests/support/http_handlers/headers.py?content=somecontent"
        ),
        fetch_post_data="somedata",
        data_types=["request", "response"],
    )
    data = await bidi_session.network.get_data(request=request, data_type="request")
    assert data["type"] == "string"
    assert data["value"] == "somedata"

    data = await bidi_session.network.get_data(request=request, data_type="response")
    assert data["type"] == "string"
    assert data["value"] == "somecontent"


async def test_request_and_response_different_collectors(
    bidi_session, url, add_data_collector, setup_collected_data
):
    request_collector = await add_data_collector(
        collector_type="blob", data_types=["request"], max_encoded_data_size=1000
    )
    [request, response_collector] = await setup_collected_data(
        fetch_url=url(
            "/webdriver/tests/support/http_handlers/headers.py?content=somecontent"
        ),
        fetch_post_data="somedata",
        data_types=["response"],
    )

    # Check that both request and response data can be retrieved
    data = await bidi_session.network.get_data(request=request, data_type="request")
    assert data["type"] == "string"
    assert data["value"] == "somedata"

    data = await bidi_session.network.get_data(request=request, data_type="response")
    assert data["type"] == "string"
    assert data["value"] == "somecontent"

    # Check that the request data can only be retrieved with the request collector
    await bidi_session.network.get_data(
        request=request, data_type="request", collector=request_collector
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="request", collector=response_collector
        )

    # Check that the response data can only be retrieved with the response collector
    await bidi_session.network.get_data(
        request=request, data_type="response", collector=response_collector
    )
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(
            request=request, data_type="response", collector=request_collector
        )

    # Remove the response collector
    await bidi_session.network.remove_data_collector(collector=response_collector)

    # Check request data can still be retrieved, but not the response data.
    await bidi_session.network.get_data(request=request, data_type="request")
    with pytest.raises(error.NoSuchNetworkDataException):
        await bidi_session.network.get_data(request=request, data_type="response")
