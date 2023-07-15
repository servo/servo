import pytest
from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException

from ... import any_int, any_string, recursive_compare
from .. import any_stack_trace, PRIMITIVE_VALUES, REMOTE_VALUES


@pytest.mark.asyncio
@pytest.mark.parametrize("expression, expected", PRIMITIVE_VALUES + REMOTE_VALUES)
async def test_exception_details(bidi_session, top_context, expression,
                                 expected):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression=f"throw {expression}",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
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


@pytest.mark.asyncio
async def test_invalid_script(bidi_session, top_context):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression="))) !!@@## some invalid JS script (((",
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )
    recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": {"type": "error"},
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        exception.value.result,
    )


@pytest.mark.asyncio
@pytest.mark.parametrize("chained", [True, False])
async def test_rejected_promise(bidi_session, top_context, chained):
    if chained:
        expression = "Promise.reject('error').then(() => { })"
    else:
        expression = "Promise.reject('error')"

    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression=expression,
            await_promise=True,
            target=ContextTarget(top_context["context"]),
        )

    recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": {"type": "string", "value": "error"},
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        exception.value.result,
    )
