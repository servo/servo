import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


async def test_params_request_invalid_phase(setup_blocked_request, bidi_session):
    request = await setup_blocked_request("authRequired")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.fail_request(request=request)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.fail_request(request=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_request_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.fail_request(request=value)


async def test_params_request_no_such_request(
    bidi_session, setup_network_test, wait_for_event, fetch, url
):
    await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    text_url = url(PAGE_EMPTY_TEXT)
    await fetch(text_url)

    response_completed_event = await on_response_completed
    request = response_completed_event["request"]["request"]

    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.fail_request(request=request)
