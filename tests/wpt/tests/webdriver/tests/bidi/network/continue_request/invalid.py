# META: timeout=long

import pytest
import webdriver.bidi.error as error

from .. import (
    create_cookie_header,
    create_header,
    PAGE_EMPTY_TEXT,
    RESPONSE_COMPLETED_EVENT,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_body_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, body=value)


@pytest.mark.parametrize("value", [{}, {"type": "string"}, {"value": "foo"}])
async def test_params_body_invalid_value(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, body=value)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_body_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request, body={"type": value, "value": "foo"}
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_body_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request, body={"type": value, "value": "foo"}
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_body_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request, body={"type": "string", "value": value}
        )


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_cookies_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, cookies=value)


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_cookies_cookie_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, cookies=[value])


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_name_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(overrides={"name": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_cookies_cookie_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [{}, {"type": "string"}, {"value": "foo"}])
async def test_params_cookies_cookie_value_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_value_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_cookies_cookie_value_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_value_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            cookies=[create_cookie_header(value_overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_headers_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, headers=value)


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_headers_header_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, headers=[value])


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_name_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(overrides={"name": value})],
        )


@pytest.mark.parametrize("value", ["", "\u0000", "\"", "{","\u0080"])
async def test_params_headers_header_name_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(overrides={"name": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_headers_header_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [{}, {"type": "string"}, {"value": "foo"}])
async def test_params_headers_header_value_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_value_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_headers_header_value_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_value_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(value_overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [" a", "a ", "\ta", "a\t", "a\nb", "a\0b"])
async def test_params_headers_header_value_value_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request,
            headers=[create_header(value_overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_method_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, method=value)


@pytest.mark.parametrize("value", ["", "\u0000", "\"", "{","\u0080"])
async def test_params_method_invalid_value(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, method=value)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_request_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.continue_request(request=value)


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
        await bidi_session.network.continue_request(request=request)


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_url_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(request=request, url=value)


@pytest.mark.parametrize("protocol", ["http", "https"])
@pytest.mark.parametrize("value", [":invalid", "#invalid"])
async def test_params_url_invalid_value(
    setup_blocked_request, bidi_session, protocol, value
):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_request(
            request=request, url=f"{protocol}://{value}"
        )
