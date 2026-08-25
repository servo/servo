import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_intercept_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.remove_intercept(intercept=value)


@pytest.mark.parametrize("value", ["foo"])
async def test_params_intercept_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchInterceptException):
        await bidi_session.network.remove_intercept(intercept=value)


async def test_params_intercept_removed_intercept(bidi_session, add_intercept):
    intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": "https://example.com"}],
    )

    await bidi_session.network.remove_intercept(intercept=intercept)

    with pytest.raises(error.NoSuchInterceptException):
        await bidi_session.network.remove_intercept(intercept=intercept)
