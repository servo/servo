import pytest

from tests.support.image import png_dimensions

from . import get_physical_document_dimensions, get_physical_viewport_dimensions


@pytest.mark.asyncio
async def test_origin(bidi_session, top_context, inline):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<div style='margin-top:2000px'>foo</div>"),
        wait="complete",
    )

    viewport_dimensions = await get_physical_viewport_dimensions(
        bidi_session, top_context
    )
    document_dimensions = await get_physical_document_dimensions(
        bidi_session, top_context
    )
    assert not viewport_dimensions == document_dimensions

    document_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], origin="document"
    )
    viewport_screenshot = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], origin="viewport"
    )

    assert png_dimensions(document_screenshot) == document_dimensions
    assert png_dimensions(viewport_screenshot) == viewport_dimensions


@pytest.mark.asyncio
@pytest.mark.parametrize("origin", ["document", "viewport"])
async def test_origin_consistency(bidi_session, top_context, inline, origin):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<div style='margin-top:2000px'>foo</div>"),
        wait="complete",
    )
    screenshot_a = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], origin=origin
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline("<div style='margin-top:2000px'>foo</div>"),
        wait="complete",
    )
    screenshot_b = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], origin=origin
    )

    assert screenshot_a == screenshot_b
