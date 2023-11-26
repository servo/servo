import pytest

import webdriver.bidi.error as error


pytestmark = pytest.mark.asyncio


MAX_INT = 9007199254740991
MIN_INT = -MAX_INT


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.traverse_history(
            context=value,
            delta=1
        )


@pytest.mark.parametrize(
    "value", [None, False, "foo", 1.5, MIN_INT - 1, MAX_INT + 1, {}, []]
)
async def test_params_delta_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.traverse_history(
            context=top_context["context"], delta=value
        )
