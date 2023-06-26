import pytest

from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException
from ... import any_int, any_string, recursive_compare
from .. import any_stack_trace, specific_error_response


@pytest.mark.asyncio
async def test_strict_mode(bidi_session, top_context):
    # As long as there is no `SOME_VARIABLE`, the command should fail in strict mode.
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression="'use strict';SOME_VARIABLE=1",
            target=ContextTarget(top_context["context"]),
            await_promise=True)
    recursive_compare(specific_error_response({"type": "error"}), exception.value.result)

    # In non-strict mode, the command should succeed and global `SOME_VARIABLE` should be created.
    result = await bidi_session.script.evaluate(
        expression="SOME_VARIABLE=1",
        target=ContextTarget(top_context["context"]),
        await_promise=True)
    assert result == {
        "type": "number",
        "value": 1}

    # Access created by the previous command `SOME_VARIABLE`.
    result = await bidi_session.script.evaluate(
        expression="'use strict';SOME_VARIABLE=1",
        target=ContextTarget(top_context["context"]),
        await_promise=True)
    assert result == {
        "type": "number",
        "value": 1}
