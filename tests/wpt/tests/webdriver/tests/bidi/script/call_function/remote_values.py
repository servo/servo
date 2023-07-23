import pytest

from webdriver.bidi.modules.script import ContextTarget, SerializationOptions
from ... import recursive_compare
from .. import REMOTE_VALUES

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("await_promise", [True, False])
@pytest.mark.parametrize("expression, expected", [
    remote_value
    for remote_value in REMOTE_VALUES if remote_value[1]["type"] != "promise"
])
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


@pytest.mark.parametrize("await_promise", [True, False])
async def test_remote_value_promise(bidi_session, top_context, await_promise):
    result = await bidi_session.script.call_function(
        function_declaration="()=>Promise.resolve(42)",
        await_promise=await_promise,
        target=ContextTarget(top_context["context"]),
    )

    if await_promise:
        assert result == {"type": "number", "value": 42}
    else:
        assert result == {"type": "promise"}
