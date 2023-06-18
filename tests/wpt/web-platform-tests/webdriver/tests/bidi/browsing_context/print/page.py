import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "page, orientation, expected_dimensions",
    [
        (None, "portrait", {"width": 21.59, "height": 27.94}),
        ({}, "portrait", {"width": 21.59, "height": 27.94}),
        ({"width": 4.5}, "portrait", {"width": 4.5, "height": 27.94}),
        ({"height": 23}, "portrait", {"width": 21.59, "height": 23}),
        ({"width": 4.5, "height": 12}, "portrait", {"width": 4.5, "height": 12}),
        ({"height": 12}, "portrait", {"width": 21.59, "height": 12}),
        (None, "landscape", {"width": 27.94, "height": 21.59}),
        ({}, "landscape", {"width": 27.94, "height": 21.59}),
        ({"width": 4.5}, "landscape", {"width": 27.94, "height": 4.5}),
        ({"height": 23}, "landscape", {"width": 23, "height": 21.59}),
        ({"width": 4.5, "height": 12}, "landscape", {"width": 12, "height": 4.5}),
        ({"height": 12}, "landscape", {"width": 12, "height": 21.59}),
    ],
)
async def test_page(
    bidi_session,
    top_context,
    inline,
    assert_pdf_dimensions,
    page,
    orientation,
    expected_dimensions,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline(""), wait="complete"
    )
    value = await bidi_session.browsing_context.print(
        context=top_context["context"], page=page, orientation=orientation
    )

    await assert_pdf_dimensions(value, expected_dimensions)
