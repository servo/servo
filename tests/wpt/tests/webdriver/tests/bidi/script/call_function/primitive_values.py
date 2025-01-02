import pytest

from webdriver.bidi.modules.script import ContextTarget
from .. import PRIMITIVE_VALUES


@pytest.mark.asyncio
@pytest.mark.parametrize("await_promise", [True, False])
@pytest.mark.parametrize("expression, expected", PRIMITIVE_VALUES)
async def test_primitive_values(bidi_session, top_context, expression,
                                expected, await_promise):
    function_declaration = f"()=>{expression}"
    if await_promise:
        function_declaration = "async" + function_declaration

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        await_promise=await_promise,
        target=ContextTarget(top_context["context"]),
    )

    assert result == expected
