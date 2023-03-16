# META: timeout=long
import pytest

pytestmark = pytest.mark.asyncio


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
                height: 27.94cm;
                {css}
            }}
        </style>
    """


@pytest.mark.parametrize(
    "margin, reference_css, css",
    [
        (
            {"top": 2.54},
            "margin-top: 1.54cm;",
            "",
        ),
        (
            {"left": 2.54},
            "margin-left: 1.54cm;",
            "",
        ),
        (
            {"right": 2.54},
            "margin-right: 1.54cm;",
            "",
        ),
        (
            {"bottom": 2.54},
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
        context=top_context["context"], url=default_content_page, wait="complete"
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
        {"top": 27.94},
        {"left": 21.59},
        {"right": 21.59},
        {"bottom": 27.94},
        {"top": 27.94, "left": 21.59, "right": 21.59, "bottom": 27.94},
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
    assert_pdf_content,
    margin,
):
    page = inline("Text")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page, wait="complete"
    )
    print_value = await bidi_session.browsing_context.print(
        context=top_context["context"],
        shrink_to_fit=False,
        margin=margin,
    )

    # Check that content is out of page dimensions.
    await assert_pdf_content(print_value, [{"type": "string", "value": ""}])


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

    # Check that margins don't affect page dimencions and equal in this case defaults.
    await assert_pdf_dimensions(value, {"width": 21.59, "height": 27.94})
