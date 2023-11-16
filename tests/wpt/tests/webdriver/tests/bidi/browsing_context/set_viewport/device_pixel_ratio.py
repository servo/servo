import pytest

from ... import get_device_pixel_ratio, get_viewport_dimensions


@pytest.mark.asyncio
@pytest.mark.parametrize("device_pixel_ratio", [1, 2])
async def test_device_pixel_ratio_only(bidi_session, new_tab, device_pixel_ratio):
    viewport = await get_viewport_dimensions(bidi_session, new_tab)

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=device_pixel_ratio)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == device_pixel_ratio
    assert await get_viewport_dimensions(bidi_session, new_tab) == viewport


@pytest.mark.asyncio
@pytest.mark.parametrize("device_pixel_ratio", [1, 2])
async def test_device_pixel_ratio_with_viewport(bidi_session, new_tab, device_pixel_ratio):
    test_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport=test_viewport,
        device_pixel_ratio=device_pixel_ratio)

    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
    assert await get_device_pixel_ratio(bidi_session, new_tab) == device_pixel_ratio


@pytest.mark.asyncio
async def test_reset_device_pixel_ratio(bidi_session, new_tab):
    original_dpr = await get_device_pixel_ratio(bidi_session, new_tab)
    test_dpr = original_dpr + 1

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=test_dpr)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == test_dpr

    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        device_pixel_ratio=None)

    assert await get_device_pixel_ratio(bidi_session, new_tab) == original_dpr
