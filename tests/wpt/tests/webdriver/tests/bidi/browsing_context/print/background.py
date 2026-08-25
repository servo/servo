import base64
import pytest

from tests.support.asserts import assert_pdf
from tests.support.image import pt_to_cm

pytestmark = pytest.mark.asyncio

INLINE_BACKGROUND_RENDERING_TEST_CONTENT = """
<style>
:root {
    background-color: black;
}
</style>
"""

BLACK_DOT_PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2NgYGD4DwABBAEAwS2OUAAAAABJRU5ErkJggg=="
WHITE_DOT_PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVQIW2P4DwQACfsD/Z8fLAAAAAAASUVORK5CYII="


@pytest.mark.parametrize("print_with_background, expected_image", [
    (None, WHITE_DOT_PNG),
    (True, BLACK_DOT_PNG),
    (False, WHITE_DOT_PNG),
], ids=["default", "true", "false"])
async def test_background(
    bidi_session,
    top_context,
    inline,
    compare_png_bidi,
    render_pdf_to_png_bidi,
    print_with_background,
    expected_image,
):
    page = inline(INLINE_BACKGROUND_RENDERING_TEST_CONTENT)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page, wait="complete")

    print_value = await bidi_session.browsing_context.print(
        context=top_context["context"],
        background=print_with_background,
        margin={
            "top": 0,
            "bottom": 0,
            "right": 0,
            "left": 0
        },
        page={
            "width": pt_to_cm(1),
            "height": pt_to_cm(1),
        },
    )

    assert_pdf(print_value)

    png = await render_pdf_to_png_bidi(print_value)
    comparison = await compare_png_bidi(png, base64.b64decode(expected_image))
    assert comparison.equal()
