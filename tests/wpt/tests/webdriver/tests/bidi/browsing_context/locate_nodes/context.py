import pytest
import webdriver.bidi.error as error


@pytest.mark.asyncio
async def test_params_context_invalid_value(bidi_session, inline, top_context):
    url = inline("""<div>foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.locate_nodes(
            context="foo", locator={ "type": "css", "value": "div" }
        )
