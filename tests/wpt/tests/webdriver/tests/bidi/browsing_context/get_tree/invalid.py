import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, "foo", {}, []])
async def test_params_max_depth_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.get_tree(max_depth=value)


@pytest.mark.parametrize("value", [-1, 1.1, 2**53])
async def test_params_max_depth_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.get_tree(max_depth=value)


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_root_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.get_tree(root=value)


async def test_params_root_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.get_tree(root="foo")
