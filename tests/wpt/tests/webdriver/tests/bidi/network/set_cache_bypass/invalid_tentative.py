import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, "foo", 42, {}, []])
async def test_params_bypass_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.set_cache_bypass(bypass=value)


@pytest.mark.parametrize("value", ["foo", 42, False, {}])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.set_cache_bypass(bypass=True, contexts=value)


async def test_params_contexts_invalid_value_empty_array(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.set_cache_bypass(bypass=True, contexts=[])


@pytest.mark.parametrize("value", [None, 42, False, {}, []])
async def test_params_contexts_invalid_array_element_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.set_cache_bypass(bypass=True, contexts=[value])


async def test_params_contexts_invalid_array_element_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.network.set_cache_bypass(bypass=True, contexts=["foo"])
