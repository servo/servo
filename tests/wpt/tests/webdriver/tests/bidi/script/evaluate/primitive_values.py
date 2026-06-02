import pytest
from webdriver.bidi.modules.script import ContextTarget
from .. import PRIMITIVE_VALUES


@pytest.mark.asyncio
@pytest.mark.parametrize("expression, expected", PRIMITIVE_VALUES)
async def test_primitive_values(bidi_session, top_context, expression,
                                expected):
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    assert result == expected
