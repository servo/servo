import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio

PAGE_EMPTY = "/webdriver/tests/bidi/browsing_context/navigate/support/empty.html"
PAGE_EMPTY_WITH_HASH_FOO = f"{PAGE_EMPTY}#foo"
PAGE_OTHER = "/webdriver/tests/bidi/browsing_context/navigate/support/other.html"


@pytest.mark.parametrize(
    "hash_before, hash_after",
    [
        ("", "#foo"),
        ("#foo", "#bar"),
        ("#foo", "#foo"),
        ("#bar", ""),
    ],
    ids=[
        "without hash to with hash",
        "with different hashes",
        "with identical hashes",
        "with hash to without hash",
    ],
)
async def test_navigate_in_the_same_document(
    bidi_session, new_tab, url, hash_before, hash_after
):
    await navigate_and_assert(bidi_session, new_tab, url(PAGE_EMPTY + hash_before))
    await navigate_and_assert(bidi_session, new_tab, url(PAGE_EMPTY + hash_after))


@pytest.mark.parametrize(
    "url_before, url_after",
    [
        (PAGE_EMPTY_WITH_HASH_FOO, f"{PAGE_OTHER}#foo"),
        (PAGE_EMPTY_WITH_HASH_FOO, f"{PAGE_OTHER}#bar"),
    ],
    ids=[
        "with identical hashes",
        "with different hashes",
    ],
)
async def test_navigate_different_documents(
    bidi_session, new_tab, url, url_before, url_after
):
    await navigate_and_assert(bidi_session, new_tab, url(url_before))
    await navigate_and_assert(bidi_session, new_tab, url(url_after))


async def test_navigate_in_iframe(bidi_session, inline, new_tab):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>")
    contexts = await navigate_and_assert(bidi_session, new_tab, url_before)

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    url_after = f"{frame_start_url}#foo"
    await navigate_and_assert(bidi_session, frame, url_after)
