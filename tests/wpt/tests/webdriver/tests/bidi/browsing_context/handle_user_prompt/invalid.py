import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.handle_user_prompt(context=value)


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_context_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.handle_user_prompt(context=value)


@pytest.mark.parametrize("value", ["foo", 42, {}, []])
async def test_params_accept_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.handle_user_prompt(
            context=top_context["context"], accept=value
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_user_text_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.handle_user_prompt(
            context=top_context["context"], user_text=value
        )


async def test_no_alert(bidi_session, top_context):
    with pytest.raises(error.NoSuchAlertException):
        await bidi_session.browsing_context.handle_user_prompt(
            context=top_context["context"]
        )
