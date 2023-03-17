import pytest

pytestmark = pytest.mark.asyncio


async def test_context(bidi_session, top_context, inline, assert_pdf_content):
    text = "Test"
    url = inline(text)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    value = await bidi_session.browsing_context.print(context=top_context["context"])

    await assert_pdf_content(value, [{"type": "string", "value": text}])


async def test_page_with_iframe(
    bidi_session, top_context, inline, iframe, assert_pdf_content
):
    text = "Test"
    iframe_content = "Iframe"
    url = inline(f"{text}<br/>{iframe(iframe_content)}")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    whole_page_value = await bidi_session.browsing_context.print(
        context=top_context["context"]
    )

    await assert_pdf_content(
        whole_page_value, [{"type": "string", "value": text + iframe_content}]
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context = contexts[0]["children"][0]

    frame_value = await bidi_session.browsing_context.print(
        context=frame_context["context"]
    )

    await assert_pdf_content(frame_value, [{"type": "string", "value": iframe_content}])


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_context_origin(
    bidi_session, top_context, inline, iframe, assert_pdf_content, domain
):
    iframe_content = "Iframe"
    url = inline(f"{iframe(iframe_content, domain=domain)}")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context = contexts[0]["children"][0]

    value = await bidi_session.browsing_context.print(context=frame_context["context"])

    await assert_pdf_content(value, [{"type": "string", "value": iframe_content}])
