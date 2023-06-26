import pytest
from webdriver.bidi.error import InvalidArgumentException, NoSuchFrameException


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, True, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.release_actions(context=value)


async def test_params_contexts_value_invalid_value(bidi_session):
    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.release_actions(context="foo")
