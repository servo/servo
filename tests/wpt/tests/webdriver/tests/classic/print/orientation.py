import pytest

from tests.support.asserts import assert_pdf, assert_success
from tests.support.image import png_dimensions

from . import do_print


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
def test_orientation(
    session,
    inline,
    render_pdf_to_png_http,
    orientation_value,
    is_portrait,
):
    session.url = inline("")

    print_result = do_print(
        session,
        {
            "orientation": orientation_value
        },
    )
    print_value = assert_success(print_result)
    assert_pdf(print_value)

    png = render_pdf_to_png_http(print_value)
    width, height = png_dimensions(png)

    assert (width < height) == is_portrait
