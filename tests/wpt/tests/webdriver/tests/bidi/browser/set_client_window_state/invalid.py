import pytest
import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases

pytestmark = pytest.mark.asyncio
MAX_INT = 9007199254740991


@pytest.mark.parametrize("invalid_id", get_invalid_cases("string"))
async def test_client_window_id_invalid_type(bidi_session, invalid_id):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=invalid_id, state="normal"
        )


async def test_client_window_id_invalid_value(bidi_session):
    with pytest.raises(error.UnknownErrorException):
        await bidi_session.browser.set_client_window_state(
            client_window="nonexistent_window", state="normal"
        )


@pytest.mark.parametrize("invalid_state", get_invalid_cases("string"))
async def test_client_window_state_invalid_type(
    bidi_session, top_context, invalid_state
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state=invalid_state
        )


async def test_client_window_state_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="invalid_state"
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_client_window_x_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", x=value
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_client_window_y_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", y=value
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_client_window_width_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", width=value
        )


@pytest.mark.parametrize("bound", [-1, MAX_INT + 1])
async def test_client_window_width_invalid_bounds(bidi_session, top_context, bound):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", width=bound
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_client_window_height_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", height=value
        )


@pytest.mark.parametrize("bound", [-1, MAX_INT + 1])
async def test_client_window_height_invalid_bounds(bidi_session, top_context, bound):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"], state="normal", height=bound
        )
