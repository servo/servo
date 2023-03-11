from base64 import decodebytes

import pytest

from . import assert_pdf
from ... import recursive_compare

pytestmark = pytest.mark.asyncio


async def test_context(bidi_session, top_context, inline, get_pdf_content):
    text = "Test"
    url = inline(text)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    value = await bidi_session.browsing_context.print(context=top_context["context"])
    pdf = decodebytes(value.encode())

    assert_pdf(pdf)

    pdf_content = await get_pdf_content(value)
    recursive_compare(
        {"type": "array", "value": [{"type": "string", "value": text}]}, pdf_content
    )
