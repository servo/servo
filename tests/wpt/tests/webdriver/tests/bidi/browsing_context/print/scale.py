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
                {css}
            }}
        </style>
    """


@pytest.mark.parametrize(
    "scale, reference_css",
    [
        (None, "width: 100px; height: 100px;"),
        (2, "width: 200px; height: 200px;"),
        (0.5, "width: 50px; height: 50px;"),
    ],
    ids=["default", "twice", "half"],
)
async def test_scale(
    bidi_session,
    top_context,
    inline,
    assert_pdf_image,
    scale,
    reference_css,
):
    not_scaled_content = get_content("width: 100px; height: 100px;")
    default_content_page = inline(not_scaled_content)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=default_content_page, wait="complete"
    )

    scaled_print_value = await bidi_session.browsing_context.print(
        context=top_context["context"],
        shrink_to_fit=False,
        scale=scale,
        background=True,
    )

    # Check that pdf scaled with print command is equal pdf of scaled with css content.
    await assert_pdf_image(scaled_print_value, get_content(reference_css), True)
    # If scale is not None, check that pdf scaled with print command is not equal pdf with not scaled content.
    if scale is not None:
        await assert_pdf_image(scaled_print_value, not_scaled_content, False)
