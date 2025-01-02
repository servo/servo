import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio

PAGE_EMPTY = "/webdriver/tests/bidi/browsing_context/support/empty.html"
PNG_BLACK_DOT = "/webdriver/tests/bidi/browsing_context/support/black_dot.png"
PNG_RED_DOT = "/webdriver/tests/bidi/browsing_context/support/red_dot.png"
SVG = "/webdriver/tests/bidi/browsing_context/support/other.svg"


@pytest.mark.parametrize(
    "url_before, url_after",
    [
        (PAGE_EMPTY, SVG),
        (SVG, PAGE_EMPTY),
        (PAGE_EMPTY, PNG_BLACK_DOT),
        (PNG_BLACK_DOT, PNG_RED_DOT),
        (PNG_RED_DOT, SVG),
        (PNG_BLACK_DOT, PAGE_EMPTY),
    ],
    ids=[
        "document to svg",
        "svg to document",
        "document to png",
        "png to png",
        "png to svg",
        "png to document",
    ],
)
async def test_navigate_between_img_and_html(
    bidi_session, new_tab, url, url_before, url_after
):
    await navigate_and_assert(bidi_session, new_tab, url(url_before))
    await navigate_and_assert(bidi_session, new_tab, url(url_after))


@pytest.mark.parametrize(
    "img",
    [SVG, PNG_BLACK_DOT],
    ids=[
        "to svg",
        "to png",
    ],
)
async def test_navigate_in_iframe(bidi_session, new_tab, inline, url, img):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>")
    contexts = await navigate_and_assert(bidi_session, new_tab, url_before)

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    await navigate_and_assert(bidi_session, frame, url(img))
