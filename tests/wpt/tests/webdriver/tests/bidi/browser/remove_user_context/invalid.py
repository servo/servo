import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_user_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.remove_user_context(user_context=value)


async def test_params_user_context_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.remove_user_context(user_context="default")


async def test_params_user_context_no_such_user_context(bidi_session):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.browser.remove_user_context(user_context="foo")


async def params_user_context_removed_user_context(bidi_session):
    user_context = await bidi_session.browser.create_user_context()
    await bidi_session.browser.remove_user_context(user_context=user_context)

    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.browser.remove_user_context(user_context=user_context)
