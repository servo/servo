from decimal import Decimal, ROUND_HALF_UP

import pytest

from tests.support.image import png_dimensions
from webdriver.bidi.modules.browsing_context import BoxOptions, ImageSizeOptions

pytestmark = pytest.mark.asyncio

TEST_PAGE = """
<style>
  html, body { margin: 0; }
  div {
    width: 200vw;
    height: 200vh;
    background: linear-gradient(to bottom right, red, blue);
  }
</style>
<div></div>
"""


def round_number(value):
    return int(Decimal(value).to_integral(rounding=ROUND_HALF_UP))


async def get_screenshot_dimensions(
    bidi_session, context, image_size=None, clip=None, origin=None
):
    data = await bidi_session.browsing_context.capture_screenshot(
        context=context, image_size=image_size, clip=clip, origin=origin
    )
    return png_dimensions(data)


async def test_max_width(bidi_session, top_context, inline):
    context = top_context["context"]
    await bidi_session.browsing_context.navigate(
        context=context, url=inline(TEST_PAGE), wait="complete"
    )
    width, height = await get_screenshot_dimensions(
        bidi_session, context, origin="document"
    )
    max_width = width // 2

    dimensions = await get_screenshot_dimensions(
        bidi_session,
        context,
        ImageSizeOptions(max_width=max_width),
        origin="document",
    )

    assert dimensions == (max_width, round_number(height * max_width / width))


async def test_max_height(bidi_session, top_context, inline):
    context = top_context["context"]
    await bidi_session.browsing_context.navigate(
        context=context, url=inline(TEST_PAGE), wait="complete"
    )
    width, height = await get_screenshot_dimensions(
        bidi_session, context, origin="document"
    )
    max_height = height // 2

    dimensions = await get_screenshot_dimensions(
        bidi_session,
        context,
        ImageSizeOptions(max_height=max_height),
        origin="document",
    )

    assert dimensions == (round_number(width * max_height / height), max_height)


async def test_max_width_and_height(bidi_session, top_context, inline):
    context = top_context["context"]
    await bidi_session.browsing_context.navigate(
        context=context, url=inline(TEST_PAGE), wait="complete"
    )
    width, height = await get_screenshot_dimensions(
        bidi_session, context, origin="document"
    )
    max_width = width // 2
    max_height = height // 4

    dimensions = await get_screenshot_dimensions(
        bidi_session,
        context,
        ImageSizeOptions(max_height=max_height, max_width=max_width),
        origin="document",
    )

    assert dimensions == (round_number(width * max_height / height), max_height)


async def test_with_clip(bidi_session, top_context):
    dimensions = await get_screenshot_dimensions(
        bidi_session,
        top_context["context"],
        ImageSizeOptions(max_width=50),
        clip=BoxOptions(x=0, y=0, width=100, height=80),
    )

    assert dimensions == (50, 40)


async def test_does_not_upscale(bidi_session, top_context):
    context = top_context["context"]
    width, height = await get_screenshot_dimensions(bidi_session, context)
    image_size = ImageSizeOptions(max_height=height + 1, max_width=width + 1)

    dimensions = await get_screenshot_dimensions(bidi_session, context, image_size)

    assert dimensions == (width, height)
