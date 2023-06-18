import base64

import pytest

from tests.support.asserts import assert_pdf, assert_success
from tests.support.image import px_to_cm

from . import do_print


INLINE_BACKGROUND_RENDERING_TEST_CONTENT = """
<style>
:root {
    background-color: black;
}
</style>
"""

BLACK_DOT_PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2NgYGD4DwABBAEAwS2OUAAAAABJRU5ErkJggg=="
WHITE_DOT_PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVQIW2P4DwQACfsD/Z8fLAAAAAAASUVORK5CYII="


@pytest.mark.parametrize(
    "print_with_background, expected_image",
    [
        (None, WHITE_DOT_PNG),
        (True, BLACK_DOT_PNG),
        (False, WHITE_DOT_PNG),
    ],
)
def test_background(
    session,
    inline,
    compare_png_http,
    render_pdf_to_png_http,
    print_with_background,
    expected_image,
):
    session.url = inline(INLINE_BACKGROUND_RENDERING_TEST_CONTENT)

    print_result = do_print(
        session,
        {
            "background": print_with_background,
            "margin": {"top": 0, "bottom": 0, "right": 0, "left": 0},
            "page": {"width": px_to_cm(1), "height": px_to_cm(1)},
        },
    )
    print_value = assert_success(print_result)
    assert_pdf(print_value)

    png = render_pdf_to_png_http(
        print_value
    )
    comparison = compare_png_http(
        png, base64.b64decode(expected_image)
    )
    assert comparison.equal()
