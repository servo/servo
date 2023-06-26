import pytest

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
async def test_arrow_function(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="()=>{return 1+2;}",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": 3}
