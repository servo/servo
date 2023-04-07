import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("script", [None, False, 42, {}, []])
async def test_params_script_invalid_type(bidi_session, script):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.remove_preload_script(script=script),


async def test_params_script_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchScriptException):
        await bidi_session.script.remove_preload_script(script="foo"),
