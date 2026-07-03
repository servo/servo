import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_screencast_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.stop_screencast(screencast=value)


async def test_params_screencast_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchScreencastException):
        await bidi_session.browsing_context.stop_screencast(screencast="_invalid_")
