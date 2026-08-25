import pytest

from tests.support.asserts import assert_pdf
from tests.support.image import png_dimensions


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "orientation_value, is_portrait",
    [
        (None, True),
        ("portrait", True),
        ("landscape", False),
    ],
    ids=[
        "default",
        "portrait",
        "landscape",
    ],
)
async def test_orientation(
    bidi_session,
    top_context,
    inline,
    render_pdf_to_png_bidi,
    orientation_value,
    is_portrait,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline(""), wait="complete"
    )
    print_value = await bidi_session.browsing_context.print(
        context=top_context["context"], orientation=orientation_value
    )

    assert_pdf(print_value)

    png = await render_pdf_to_png_bidi(print_value)
    width, height = png_dimensions(png)

    assert (width < height) == is_portrait
