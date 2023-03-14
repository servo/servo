import pytest

from tests.support.image import png_dimensions

from . import viewport_dimensions


@pytest.mark.asyncio
async def test_capture(bidi_session, url, top_context, inline, compare_png_bidi):
    expected_size = await viewport_dimensions(bidi_session, top_context)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url="about:blank", wait="complete"
    )
    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])
    assert png_dimensions(reference_data) == expected_size

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])

    comparison = await compare_png_bidi(data, reference_data)
    assert not comparison.equal()

    # Take a second screenshot that should be identical to validate that
    # we don't just always return false here
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])

    comparison = await compare_png_bidi(new_data, data)
    assert comparison.equal()
