import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("context", [False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.get_realms(context=context)


async def test_params_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.get_realms(context="foo")


@pytest.mark.parametrize("type", [False, 42, {}, []])
async def test_params_type_invalid_type(bidi_session, type):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.get_realms(type=type)


async def test_params_type_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.get_realms(type="foo")
