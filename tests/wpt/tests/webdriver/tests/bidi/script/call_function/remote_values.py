import pytest

from webdriver.bidi.modules.script import ContextTarget, SerializationOptions
from ... import recursive_compare
from .. import REMOTE_VALUES

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("await_promise", [True, False])
@pytest.mark.parametrize("expression, expected", REMOTE_VALUES)
async def test_remote_values(bidi_session, top_context, await_promise,
                             expression, expected):
    function_declaration = f"()=>{expression}"
    if await_promise:
        function_declaration = "async" + function_declaration

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        await_promise=await_promise,
        target=ContextTarget(top_context["context"]),
        serialization_options=SerializationOptions(max_object_depth=1),
    )

    recursive_compare(expected, result)


async def test_remote_value_promise_await(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="()=>Promise.resolve(42)",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": 42}


async def test_remote_value_promise_no_await(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="()=>Promise.resolve(42)",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "promise"}
