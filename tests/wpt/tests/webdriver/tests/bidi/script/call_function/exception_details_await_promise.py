import pytest
from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException

from ... import any_int, any_string, recursive_compare
from .. import any_stack_trace, PRIMITIVE_VALUES, REMOTE_VALUES


@pytest.mark.asyncio
@pytest.mark.parametrize("expression, expected", PRIMITIVE_VALUES + REMOTE_VALUES)
@pytest.mark.asyncio
async def test_exception_details(bidi_session, top_context, expression, expected):
    function_declaration = f"async()=>{{ throw {expression} }}"

    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.call_function(
            function_declaration=function_declaration,
            await_promise=True,
            target=ContextTarget(top_context["context"]),
        )

    recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": expected,
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        exception.value.result,
    )
