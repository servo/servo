import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


# The following tests are marked as tentative until
# https://github.com/w3c/webdriver-bidi/issues/274 is resolved.
async def test_params_target_invalid_value(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="() => 1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="() => 1 + 2",
            target={"context": top_context["context"], "realm": result["realm"]},
            await_promise=True,
        )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="() => 1 + 2",
            target={"sandbox": "foo", "realm": result["realm"]},
            await_promise=True,
        )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="() => 1 + 2",
            target={"sandbox": "bar"},
            await_promise=True,
        )
