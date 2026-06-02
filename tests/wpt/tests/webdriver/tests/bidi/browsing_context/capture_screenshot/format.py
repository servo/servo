import pytest

from webdriver.bidi.modules.browsing_context import FormatOptions


@pytest.mark.asyncio
async def test_format_type(bidi_session, top_context, inline):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<div style='margin-top:2000px'>foo</div>"),
        wait="complete")

    png_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        format=FormatOptions(type="image/png"))
    jpeg_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        format=FormatOptions(type="image/jpeg"))

    assert png_screenshot != jpeg_screenshot


@pytest.mark.asyncio
async def test_format_quality(bidi_session, top_context, inline):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<div style='margin-top:2000px'>foo</div>"),
        wait="complete")

    jpeg_quality_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        format=FormatOptions(type="image/jpeg",quality=0.1))
    jpeg_high_quality_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        format=FormatOptions(type="image/jpeg",quality=1))

    assert jpeg_quality_screenshot != jpeg_high_quality_screenshot

    assert len(jpeg_high_quality_screenshot) > len(jpeg_quality_screenshot)
