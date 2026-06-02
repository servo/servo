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


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_cookies_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, cookies=value)


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_cookies_cookie_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, cookies=[value])


@pytest.mark.parametrize(
    "value",
    [{}, {"name": "name"}, {"value": {"type": "string", "value": "foo"}}],
    ids=[
        "empty object",
        "missing value",
        "missing name",
    ],
)
async def test_params_cookies_cookie_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[value],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_name_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"name": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_cookies_cookie_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [{}, {"type": "string"}, {"value": "foo"}])
async def test_params_cookies_cookie_value_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_value_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_cookies_cookie_value_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("property", ["domain", "expiry", "path", "sameSite"])
@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_cookies_cookie_value_string_properties_invalid_type(
    setup_blocked_request, bidi_session, property, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={property: value})],
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_cookies_cookie_value_same_site_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"sameSite": value})],
        )


@pytest.mark.parametrize("property", ["httpOnly", "secure"])
@pytest.mark.parametrize("value", [42, "foo", {}, []])
async def test_params_cookies_cookie_value_bool_properties_invalid_type(
    setup_blocked_request, bidi_session, property, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={property: value})],
        )


@pytest.mark.parametrize("value", [False, "foo", {}, []])
async def test_params_cookies_cookie_value_max_age_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"maxAge": value})],
        )


@pytest.mark.parametrize("value", [4.3])
async def test_params_cookies_cookie_value_max_age_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(overrides={"maxAge": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_cookies_cookie_value_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            cookies=[create_cookie_header(value_overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_credentials_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, credentials=value)


@pytest.mark.parametrize(
    "value",
    [
        {"type": "password", "password": "foo"},
        {"type": "password", "username": "foo"},
        {
            "type": "password",
        },
        {
            "username": "foo",
            "password": "bar",
        },
    ],
    ids=[
        "missing username",
        "missing password",
        "missing username and password",
        "missing type",
    ],
)
async def test_params_credentials_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, credentials=value)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_credentials_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            credentials={
                "type": value,
            },
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_credentials_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            credentials={
                "type": value,
            },
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_credentials_username_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")
    credentials = {"type": "password", "username": value, "password": "foo"}
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request, credentials=credentials
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_credentials_password_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")
    credentials = {"type": "password", "username": "foo", "password": value}
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request, credentials=credentials
        )


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_headers_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, headers=value)


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_headers_header_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, headers=[value])


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_name_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(overrides={"name": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_headers_header_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [{}, {"type": "string"}, {"value": "foo"}])
async def test_params_headers_header_value_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(overrides={"value": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_value_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_headers_header_value_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(value_overrides={"type": value})],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_headers_header_value_value_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request,
            headers=[create_header(value_overrides={"value": value})],
        )


async def test_params_request_invalid_phase(setup_blocked_request, bidi_session):
    request = await setup_blocked_request("beforeRequestSent")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request)


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_request_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.continue_response(request=value)


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
        await bidi_session.network.continue_response(request=request)


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_reason_phrase_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(
            request=request, reason_phrase=value
        )


@pytest.mark.parametrize("value", [False, "foo", {}, []])
async def test_params_status_code_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, status_code=value)


@pytest.mark.parametrize("value", [-1, 4.3])
async def test_params_status_code_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("responseStarted")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_response(request=request, status_code=value)
