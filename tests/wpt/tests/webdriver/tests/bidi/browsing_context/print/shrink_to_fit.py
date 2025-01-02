import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "shrink_to_fit, pages_content",
    [
        (None, [{"type": "string", "value": "Block 1Block 2Block 3Block 4"}]),
        (True, [{"type": "string", "value": "Block 1Block 2Block 3Block 4"}]),
        (
            False,
            [
                {"type": "string", "value": "Block 1Block 2Block 3"},
                {"type": "string", "value": "Block 4"},
            ],
        ),
    ],
    ids=["default", "True", "False"],
)
async def test_shrink_to_fit(
    bidi_session,
    top_context,
    inline,
    assert_pdf_content,
    shrink_to_fit,
    pages_content,
):
    url = inline(
        """
        <style>
            div {
                width: 1200px;
                height: 400px;
            }
        </style>
        <div>Block 1</div>
        <div>Block 2</div>
        <div>Block 3</div>
        <div>Block 4</div>
    """
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    value = await bidi_session.browsing_context.print(
        context=top_context["context"], shrink_to_fit=shrink_to_fit
    )

    await assert_pdf_content(value, pages_content)
