import pytest
import webdriver.bidi.error as error

from .. import PAGE_EMPTY_TEXT, RESPONSE_COMPLETED_EVENT

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", ["beforeRequestSent", "responseStarted"])
async def test_params_request_invalid_phase(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request(value)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(request=request, action="cancel")


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_request_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(request=value, action="cancel")


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_request_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.continue_with_auth(request=value, action="cancel")


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
        await bidi_session.network.continue_with_auth(request=request, action="cancel")


async def test_params_request_no_such_request_after_cancel(
    setup_blocked_request, bidi_session, subscribe_events, wait_for_event
):
    request = await setup_blocked_request("authRequired")

    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    await bidi_session.network.continue_with_auth(request=request, action="cancel")
    await on_response_completed

    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.continue_with_auth(request=request, action="cancel")


async def test_params_request_no_such_request_after_provideCredentials(
    setup_blocked_request, bidi_session, subscribe_events, wait_for_event
):
    # Setup unique username / password because browsers cache credentials.
    username = "test_params_request_no_such_request_after_provideCredentials"
    password = "test_params_request_no_such_request_after_provideCredentials_password"
    request = await setup_blocked_request("authRequired", username=username, password=password)

    await subscribe_events(events=[RESPONSE_COMPLETED_EVENT])
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    credentials = {
        "type": "password",
        "username": username,
        "password": password,
    }
    await bidi_session.network.continue_with_auth(
        request=request, action="provideCredentials", credentials=credentials
    )
    await on_response_completed

    with pytest.raises(error.NoSuchRequestException):
        await bidi_session.network.continue_with_auth(request=request, action="cancel")


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_action_invalid_type(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("authRequired")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(request=request, action=value)


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_action_invalid_value(setup_blocked_request, bidi_session, value):
    request = await setup_blocked_request("authRequired")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(request=request, action=value)


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
        None,
    ],
    ids=[
        "missing username",
        "missing password",
        "missing username and password",
        "missing type",
        "missing credentials",
    ],
)
async def test_params_action_provideCredentials_invalid_credentials(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("authRequired")

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(
            request=request, action="provideCredentials", credentials=value
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_action_provideCredentials_credentials_type_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("authRequired")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(
            request=request, action="provideCredentials", credentials={"type": value,}
        )


@pytest.mark.parametrize("value", ["", "foo"])
async def test_params_action_provideCredentials_credentials_type_invalid_value(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("authRequired")
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(
            request=request, action="provideCredentials", credentials={"type": value,}
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_action_provideCredentials_credentials_username_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("authRequired")
    credentials = {"type": "password", "username": value, "password": "foo"}
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(
            request=request, action="provideCredentials", credentials=credentials
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_action_provideCredentials_credentials_password_invalid_type(
    setup_blocked_request, bidi_session, value
):
    request = await setup_blocked_request("authRequired")
    credentials = {"type": "password", "username": "foo", "password": value}
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.continue_with_auth(
            request=request, action="provideCredentials", credentials=credentials
        )
