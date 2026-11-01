import pytest

from tests.support.image import png_dimensions

from . import get_visual_viewport_dimensions
from ... import (get_device_pixel_ratio, get_document_dimensions,
                 get_viewport_dimensions)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("activate", [True, False],
                         ids=["with activate", "without activate"])
async def test_capture(bidi_session, top_context, inline, compare_png_bidi,
                       activate):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url="about:blank", wait="complete"
    )
    if activate:
        await bidi_session.browsing_context.activate(
            context=top_context["context"])

    # origin="viewport" captures the visual viewport, excluding scrollbars.
    viewport = await get_visual_viewport_dimensions(bidi_session, top_context)
    dpr = await get_device_pixel_ratio(bidi_session, top_context)

    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])
    # Viewport dimensions scaled by DPR can be fractional, and rounding to
    # integer PNG pixel dimensions (floor, ceil, or round) is browser-defined.
    assert png_dimensions(reference_data) == pytest.approx(
        (viewport["width"] * dpr, viewport["height"] * dpr), abs=1.0
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    if activate:
        await bidi_session.browsing_context.activate(
            context=top_context["context"])
    data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])

    comparison = await compare_png_bidi(data, reference_data)
    assert not comparison.equal()

    # Take a second screenshot that should be identical to validate that
    # we don't just always return false here
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    if activate:
        await bidi_session.browsing_context.activate(
            context=top_context["context"])
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])

    comparison = await compare_png_bidi(new_data, data)
    assert comparison.equal()


@pytest.mark.parametrize("delta_width", [-10, +20], ids=["width smaller", "width larger"])
@pytest.mark.parametrize("delta_height", [-30, +40], ids=["height smaller", "height larger"])
async def test_capture_with_viewport(bidi_session, new_tab, delta_width, delta_height):
    original_viewport = await get_viewport_dimensions(bidi_session, new_tab)

    dpr = await get_device_pixel_ratio(bidi_session, new_tab)

    test_viewport = {
        "width": original_viewport["width"] + delta_width,
        "height": original_viewport["height"] + delta_height
    }
    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url="about:blank", wait="complete"
    )

    result = await bidi_session.browsing_context.capture_screenshot(
        context=new_tab["context"])
    assert png_dimensions(result) == pytest.approx(
        (test_viewport["width"] * dpr, test_viewport["height"] * dpr), abs=1.0
    )


@pytest.mark.parametrize("dpr", [0.5, 2])
async def test_capture_with_different_dpr(bidi_session, new_tab, inline, dpr):
    page = inline("<div style='background-color: black; width: 100px; height: 100px;'></div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page, wait="complete"
    )

    original_viewport = await get_visual_viewport_dimensions(bidi_session, new_tab)

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=dpr)

    data = await bidi_session.browsing_context.capture_screenshot(context=new_tab["context"])
    # Rounding of fractional physical pixels is implementation-defined.
    assert png_dimensions(data) == pytest.approx(
        (original_viewport["width"] * dpr, original_viewport["height"] * dpr), abs=1.0
    )


@pytest.mark.parametrize("dpr", [1, 3])
async def test_clip_huge_element_to_viewport(bidi_session, new_tab, inline,
                                             dpr):
    width = 32768
    height = 32768
    test_viewport = {"width": 250, "height": 300}

    url = inline(f"""
        <style>
            /* Hide scrollbars without constraining the huge body extent. */
            html {{ overflow: hidden; }}
            body {{ margin: 0; }}
        </style>
        <div style="width: {width}px; height: {height}px; background-color: black;"></div>
    """)
    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport,
        device_pixel_ratio=dpr,
    )

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=url,
                                                 wait="complete")

    viewport_dimensions = await get_viewport_dimensions(bidi_session, new_tab)
    document_dimensions = await get_document_dimensions(bidi_session, new_tab)
    observed_dpr = await get_device_pixel_ratio(bidi_session, new_tab)
    assert viewport_dimensions == test_viewport
    assert observed_dpr == pytest.approx(dpr)
    assert document_dimensions["width"] >= width
    assert document_dimensions["height"] >= height

    data = await bidi_session.browsing_context.capture_screenshot(
        context=new_tab["context"])
    assert png_dimensions(data) == pytest.approx(
        (viewport_dimensions["width"] * observed_dpr,
         viewport_dimensions["height"] * observed_dpr), abs=1.0
    )
