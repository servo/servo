import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.fail_request(request=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_request_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.fail_request(request=value)


async def test_params_request_no_such_request(bidi_session, setup_network_test,
                                              wait_for_event, wait_for_future_safe,
                                              fetch, url):
    await setup_network_test(events=[
        RESPONSE_COMPLETED_EVENT,
    ])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    text_url = url(PAGE_EMPTY_TEXT)
    await fetch(text_url)

    response_completed_event = await wait_for_future_safe(on_response_completed)
    request = response_completed_event["request"]["request"]

    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.fail_request(request=request)
