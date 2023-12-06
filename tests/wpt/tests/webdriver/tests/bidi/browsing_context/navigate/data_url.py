from urllib.parse import quote

import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio


def dataURL(doc, mime_type="text/html", charset="utf-8", is_base64=False):
    encoding = ""
    if charset:
        encoding = f"charset={charset}"
    elif is_base64:
        encoding = "base64"

    return f"data:{mime_type};{encoding},{quote(doc)}"


HTML_BAR = dataURL("<p>bar</p>")
HTML_FOO = dataURL("<p>foo</p>")
IMG_BLACK_PIXEL = dataURL(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
    "image/png",
    None,
    True,
)
IMG_RED_PIXEL = dataURL(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEX/TQBcNTh/AAAAAXRSTlPM0jRW/QAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII=",
    "image/png",
    None,
    True,
)
PAGE = "/webdriver/tests/bidi/browsing_context/support/empty.html"
TEXT_BAR = dataURL("bar", "text/plain")
TEXT_FOO = dataURL("foo", "text/plain")


def wrap_content_in_url(url, content):
    """Check if content is not data url and wrap it in the url function"""
    if content.startswith("data:"):
        return content
    return url(content)


@pytest.mark.parametrize(
    "url_before, url_after",
    [
        (PAGE, IMG_BLACK_PIXEL),
        (IMG_BLACK_PIXEL, IMG_RED_PIXEL),
        (IMG_BLACK_PIXEL, HTML_FOO),
        (IMG_BLACK_PIXEL, PAGE),
        (PAGE, HTML_FOO),
        (HTML_FOO, TEXT_FOO),
        (HTML_FOO, HTML_BAR),
        (HTML_FOO, PAGE),
        (PAGE, TEXT_FOO),
        (TEXT_FOO, TEXT_BAR),
        (TEXT_FOO, IMG_BLACK_PIXEL),
        (TEXT_FOO, PAGE),
    ],
    ids=[
        "document to data:image",
        "data:image to data:image",
        "data:image to data:html",
        "data:image to document",
        "document to data:html",
        "data:html to data:html",
        "data:html to data:text",
        "data:html to document",
        "document to data:text",
        "data:text to data:text",
        "data:text to data:image",
        "data:text to document",
    ],
)
async def test_navigate_from_single_page(
    bidi_session, new_tab, url, url_before, url_after
):
    await navigate_and_assert(
        bidi_session,
        new_tab,
        wrap_content_in_url(url, url_before),
    )
    await navigate_and_assert(
        bidi_session,
        new_tab,
        wrap_content_in_url(url, url_after),
    )


async def test_navigate_in_iframe(bidi_session, inline, new_tab):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>")
    contexts = await navigate_and_assert(bidi_session, new_tab, url_before)

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    await navigate_and_assert(bidi_session, frame, HTML_BAR)
