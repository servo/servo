import pytest
from webdriver.bidi.error import UnknownErrorException
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_deeply_nested_result(bidi_session, top_context):
    # A remote end can fail to serialize a value this deep, but it has to answer the command.
    try:
        result = await bidi_session.script.call_function(
            function_declaration="() => { let a = []; for (let i = 1; i < 1000; ++i) a = [a]; return a; }",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )
    except UnknownErrorException:
        return

    assert result["type"] == "array"
