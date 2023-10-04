# META: timeout=long
from math import ceil
import pytest

from webdriver.bidi.error import UnsupportedOperationException
from tests.support.image import inch_in_cm, inch_in_point

pytestmark = pytest.mark.asyncio

DEFAULT_PAGE_HEIGHT = 27.94
DEFAULT_PAGE_WIDTH = 21.59


def get_content(css=""):
    return f"""
        <div></div>
        <style>
            html,
            body {{
                margin: 0;
            }}
            div {{
                background-color: black;
                height: {DEFAULT_PAGE_HEIGHT}cm;
                {css}
            }}
        </style>
    """


@pytest.mark.parametrize(
    "margin, reference_css, css",
    [
        (
            {"top": inch_in_cm},
            "margin-top: 1.54cm;",
            "",
        ),
        (
            {"left": inch_in_cm},
            "margin-left: 1.54cm;",
            "",
        ),
        (
            {"right": inch_in_cm},
            "margin-right: 1.54cm;",
            "",
        ),
        (
            {"bottom": inch_in_cm},
            "height: 24.4cm;",
            "height: 26.94cm;",
        ),
    ],
    ids=[
        "top",
        "left",
        "right",
        "bottom",
    ],
)
async def test_margin_default(
    bidi_session,
    top_context,
    inline,
    assert_pdf_image,
    margin,
    reference_css,
    css,
):
    default_content_page = inline(get_content(css))
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=default_content_page,
        wait="complete"
    )
    value_with_margin = await bidi_session.browsing_context.print(
        context=top_context["context"],
        shrink_to_fit=False,
        background=True,
        margin=margin,
    )

    # Compare a page with default margin (1.0cm) + css margin
    # with a page with extended print margin.
    await assert_pdf_image(value_with_margin, get_content(reference_css), True)


@pytest.mark.parametrize(
    "margin",
    [
        {"top": DEFAULT_PAGE_HEIGHT},
        {"left": DEFAULT_PAGE_WIDTH},
        {"right": DEFAULT_PAGE_WIDTH},
        {"bottom": DEFAULT_PAGE_HEIGHT},
        {
            "top": DEFAULT_PAGE_HEIGHT,
            "left": DEFAULT_PAGE_WIDTH,
            "right": DEFAULT_PAGE_WIDTH,
            "bottom": DEFAULT_PAGE_HEIGHT,
        },
    ],
    ids=[
        "top",
        "left",
        "right",
        "bottom",
        "all",
    ],
)
async def test_margin_same_as_page_dimension(
    bidi_session,
    top_context,
    inline,
    margin,
):
    page = inline("Text")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page, wait="complete"
    )

    # This yields an empty content area: https://github.com/w3c/webdriver-bidi/issues/473
    with pytest.raises(UnsupportedOperationException):
        await bidi_session.browsing_context.print(
            context=top_context["context"],
            shrink_to_fit=False,
            margin=margin,
        )


@pytest.mark.parametrize(
    "margin",
    [
        {"top": DEFAULT_PAGE_HEIGHT - ceil(inch_in_cm / inch_in_point)},
        {"left": DEFAULT_PAGE_WIDTH - ceil(inch_in_cm / inch_in_point)},
        {"right": DEFAULT_PAGE_WIDTH - ceil(inch_in_cm / inch_in_point)},
        {"bottom": DEFAULT_PAGE_HEIGHT - ceil(inch_in_cm / inch_in_point)},
    ],
    ids=[
        "top",
        "left",
        "right",
        "bottom",
    ],
)
async def test_margin_minimum_page_size(
    bidi_session,
    top_context,
    inline,
    assert_pdf_dimensions,
    margin,
):
    page = inline("Text")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page, wait="complete"
    )

    value = await bidi_session.browsing_context.print(
        context=top_context["context"],
        shrink_to_fit=False,
        margin=margin
    )

    if "top" in margin or "bottom" in margin:
        expected_width = DEFAULT_PAGE_WIDTH
    else:
        expected_width = DEFAULT_PAGE_WIDTH - (inch_in_cm / inch_in_point)

    if "left" in margin or "right" in margin:
        expected_height = DEFAULT_PAGE_HEIGHT
    else:
        expected_height = DEFAULT_PAGE_HEIGHT - (inch_in_cm / inch_in_point)

    # Check that margins don't affect page dimensions and equal defaults.
    await assert_pdf_dimensions(value, {
       "width": expected_width,
       "height": expected_height,
    })


@pytest.mark.parametrize(
    "margin",
    [
        {},
        {"top": 0, "left": 0, "right": 0, "bottom": 0},
        {"top": 2, "left": 2, "right": 2, "bottom": 2}
    ],
    ids=[
        "default",
        "0",
        "2"
    ],
)
async def test_margin_does_not_affect_page_size(
    bidi_session,
    top_context,
    inline,
    assert_pdf_dimensions,
    margin
):
    url = inline("")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    value = await bidi_session.browsing_context.print(
        context=top_context["context"],
        margin=margin
    )

    # Check that margins don't affect page dimensions
    # and equal in this case defaults.
    await assert_pdf_dimensions(value, {
        "width": DEFAULT_PAGE_WIDTH,
        "height": DEFAULT_PAGE_HEIGHT,
    })
