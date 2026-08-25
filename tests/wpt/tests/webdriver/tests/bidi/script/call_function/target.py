import pytest

from webdriver.bidi.modules.script import (
    ContextTarget,
)

from ... import recursive_compare

pytestmark = pytest.mark.asyncio


async def test_target_context_and_realm(bidi_session, top_context, new_tab):
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="() => { window.foo = 3; }",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )
    realm = result["realm"]

    # Make sure that realm argument is ignored and
    # script is executed in the right context.
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="() => window.foo",
        target={"context": new_tab["context"], "realm": realm},
        await_promise=True,
    )

    assert realm != result["realm"]
    recursive_compare(
        {"realm": result["realm"], "result": {"type": "undefined"}}, result
    )
