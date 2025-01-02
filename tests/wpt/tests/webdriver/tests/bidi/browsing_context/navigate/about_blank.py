import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio

PAGE_ABOUT_BLANK = "about:blank"
PAGE_EMPTY = "/webdriver/tests/bidi/browsing_context/support/empty.html"


async def test_navigate_from_single_page(bidi_session, new_tab, url):
    await navigate_and_assert(bidi_session, new_tab, url(PAGE_EMPTY))
    await navigate_and_assert(bidi_session, new_tab, PAGE_ABOUT_BLANK)


async def test_navigate_from_frameset(bidi_session, inline, new_tab, url):
    frame_url = url(PAGE_EMPTY)
    url_before = inline(f"<frameset><frame src='{frame_url}'/></frameset")
    await navigate_and_assert(bidi_session, new_tab, url_before)

    await navigate_and_assert(bidi_session, new_tab, PAGE_ABOUT_BLANK)


async def test_navigate_in_iframe(bidi_session, inline, new_tab):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>")
    contexts = await navigate_and_assert(bidi_session, new_tab, url_before)

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    await navigate_and_assert(bidi_session, frame, PAGE_ABOUT_BLANK)
