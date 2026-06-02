import pytest
import asyncio

from webdriver.bidi.modules.script import ContextTarget

from .. import CSP_EXPRESSIONS


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression",
    CSP_EXPRESSIONS.values(),
    ids=CSP_EXPRESSIONS.keys(),
)
async def test_default_src_unsafe_inline(
    bidi_session, top_context, setup_csp_tentative_test, expression
):
    function_declaration = f"() => ({expression})"
    result = await asyncio.wait_for(
        asyncio.shield(
            bidi_session.script.call_function(
                function_declaration=function_declaration,
                target=ContextTarget(top_context["context"]),
                await_promise=True,
            )
        ),
        timeout=2.0,
    )

    assert result == {"type": "number", "value": 3}
