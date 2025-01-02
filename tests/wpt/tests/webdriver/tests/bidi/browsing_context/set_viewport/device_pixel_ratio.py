import pytest
from webdriver.bidi.modules.script import ContextTarget

from ... import get_device_pixel_ratio, get_viewport_dimensions


@pytest.mark.asyncio
@pytest.mark.parametrize("device_pixel_ratio", [0.5, 2])
async def test_device_pixel_ratio_only(bidi_session, inline, new_tab, device_pixel_ratio):
    viewport = await get_viewport_dimensions(bidi_session, new_tab)

    # Load a page so that reflow is triggered when changing the DPR
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=device_pixel_ratio)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == device_pixel_ratio
    assert await get_viewport_dimensions(bidi_session, new_tab) == viewport


@pytest.mark.asyncio
@pytest.mark.parametrize("device_pixel_ratio", [0.5, 2])
async def test_device_pixel_ratio_with_viewport(
    bidi_session, inline, new_tab, device_pixel_ratio
):
    test_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Load a page so that reflow is triggered when changing the DPR
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport,
        device_pixel_ratio=device_pixel_ratio)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
    assert await get_device_pixel_ratio(bidi_session, new_tab) == device_pixel_ratio


@pytest.mark.asyncio
async def test_reset_device_pixel_ratio(bidi_session, inline, new_tab):
    original_dpr = await get_device_pixel_ratio(bidi_session, new_tab)
    test_dpr = original_dpr + 1

    # Load a page so that reflow is triggered when changing the DPR
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=test_dpr)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == test_dpr

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=None)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == original_dpr


@pytest.mark.asyncio
@pytest.mark.parametrize("device_pixel_ratio", [0.5, 2])
@pytest.mark.parametrize(
    "use_horizontal_scrollbar, use_vertical_scrollbar",
    [
        (True, False),
        (False, True),
        (True, True),
    ],
    ids=["horizontal", "vertical", "both"],
)
async def test_device_pixel_ratio_with_scrollbar(
    bidi_session,
    inline,
    new_tab,
    device_pixel_ratio,
    use_horizontal_scrollbar,
    use_vertical_scrollbar,
):
    viewport_dimensions = await get_viewport_dimensions(bidi_session, new_tab)

    width = 100
    if use_horizontal_scrollbar:
        width = viewport_dimensions["width"] + 100

    height = 100
    if use_vertical_scrollbar:
        height = viewport_dimensions["height"] + 100

    html = f"""<div style="width: {width}px; height: {height}px;">foo</div>"""
    page_url = inline(html)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"], device_pixel_ratio=device_pixel_ratio
    )

    assert await get_device_pixel_ratio(bidi_session, new_tab) == device_pixel_ratio
    assert await get_viewport_dimensions(bidi_session, new_tab) == viewport_dimensions
