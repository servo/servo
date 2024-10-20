import pytest

pytestmark = pytest.mark.asyncio


async def test_context_with_frame(bidi_session, top_context, inline, assert_pdf_content):
    text = "Test"
    url = inline(text)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    value = await bidi_session.browsing_context.print(context=top_context["context"])

    await assert_pdf_content(value, [{"type": "string", "value": text}])
