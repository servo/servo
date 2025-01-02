import pytest

from ... import recursive_compare
from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
@pytest.mark.parametrize("await_promise", [True, False])
async def test_await_promise_delayed(bidi_session, top_context, await_promise):
    result = await bidi_session.script.call_function(
        function_declaration="""
          async function() {{
            await new Promise(r => setTimeout(() => r(), 0));
            return "SOME_DELAYED_RESULT";
          }}
        """,
        await_promise=await_promise,
        target=ContextTarget(top_context["context"]),
    )

    if await_promise:
        assert result == {
            "type": "string",
            "value": "SOME_DELAYED_RESULT"}
    else:
        recursive_compare({
            "type": "promise"},
            result)


@pytest.mark.asyncio
@pytest.mark.parametrize("await_promise", [True, False])
async def test_await_promise_async_arrow(bidi_session, top_context, await_promise):
    result = await bidi_session.script.call_function(
        function_declaration="async ()=>{return 'SOME_VALUE'}",
        await_promise=await_promise,
        target=ContextTarget(top_context["context"]))

    if await_promise:
        assert result == {
            "type": "string",
            "value": "SOME_VALUE"}
    else:
        recursive_compare({
            "type": "promise"},
            result)
