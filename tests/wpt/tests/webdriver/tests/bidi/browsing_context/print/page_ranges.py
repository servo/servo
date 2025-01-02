# META: timeout=long
import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "ranges,expected",
    [
        (
            ["2-4"],
            [
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 3"},
                {"type": "string", "value": "Page 4"},
            ],
        ),
        (
            ["2-4", "2-3"],
            [
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 3"},
                {"type": "string", "value": "Page 4"},
            ],
        ),
        (
            ["2-4", "3-5"],
            [
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 3"},
                {"type": "string", "value": "Page 4"},
                {"type": "string", "value": "Page 5"},
            ],
        ),
        (
            ["9-"],
            [
                {"type": "string", "value": "Page 9"},
                {"type": "string", "value": "Page 10"},
            ],
        ),
        (
            ["-2"],
            [
                {"type": "string", "value": "Page 1"},
                {"type": "string", "value": "Page 2"},
            ],
        ),
        (
            [7],
            [
                {"type": "string", "value": "Page 7"},
            ],
        ),
        (
            ["7"],
            [
                {"type": "string", "value": "Page 7"},
            ],
        ),
        (
            ["-2", "9-", "7"],
            [
                {"type": "string", "value": "Page 1"},
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 7"},
                {"type": "string", "value": "Page 9"},
                {"type": "string", "value": "Page 10"},
            ],
        ),
        (
            ["-5", "2-"],
            [
                {"type": "string", "value": "Page 1"},
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 3"},
                {"type": "string", "value": "Page 4"},
                {"type": "string", "value": "Page 5"},
                {"type": "string", "value": "Page 6"},
                {"type": "string", "value": "Page 7"},
                {"type": "string", "value": "Page 8"},
                {"type": "string", "value": "Page 9"},
                {"type": "string", "value": "Page 10"},
            ],
        ),
        (
            [],
            [
                {"type": "string", "value": "Page 1"},
                {"type": "string", "value": "Page 2"},
                {"type": "string", "value": "Page 3"},
                {"type": "string", "value": "Page 4"},
                {"type": "string", "value": "Page 5"},
                {"type": "string", "value": "Page 6"},
                {"type": "string", "value": "Page 7"},
                {"type": "string", "value": "Page 8"},
                {"type": "string", "value": "Page 9"},
                {"type": "string", "value": "Page 10"},
            ],
        ),
    ],
)
async def test_page_ranges_document(
    bidi_session, inline, top_context, assert_pdf_content, ranges, expected
):
    url = inline(
        """
<style>
div {page-break-after: always}
</style>

<div>Page 1</div>
<div>Page 2</div>
<div>Page 3</div>
<div>Page 4</div>
<div>Page 5</div>
<div>Page 6</div>
<div>Page 7</div>
<div>Page 8</div>
<div>Page 9</div>
<div>Page 10</div>"""
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    value = await bidi_session.browsing_context.print(
        context=top_context["context"], page_ranges=ranges
    )

    await assert_pdf_content(value, expected)
