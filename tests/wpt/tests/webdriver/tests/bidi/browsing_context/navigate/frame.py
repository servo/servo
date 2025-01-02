import pytest

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio

PAGE_CONTENT = "<div>foo</div>"


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_origin(bidi_session, new_tab, inline, domain):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>", domain=domain)
    contexts = await navigate_and_assert(bidi_session, new_tab, url_before)

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    await navigate_and_assert(bidi_session, frame, inline(PAGE_CONTENT))


async def test_multiple_frames(
    bidi_session, new_tab, test_page_multiple_frames, test_page, test_page2, inline
):
    contexts = await navigate_and_assert(
        bidi_session, new_tab, test_page_multiple_frames
    )

    assert len(contexts[0]["children"]) == 2
    frame = contexts[0]["children"][0]
    assert frame["url"] == test_page

    await navigate_and_assert(bidi_session, frame, inline(PAGE_CONTENT))

    # Make sure that the second frame hasn't been navigated
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert contexts[0]["children"][1]["url"] == test_page2


async def test_nested_frames(
    bidi_session,
    new_tab,
    inline,
    test_page_nested_frames,
    test_page_same_origin_frame,
    test_page,
):
    contexts = await navigate_and_assert(bidi_session, new_tab, test_page_nested_frames)

    assert len(contexts[0]["children"]) == 1
    frame_level_1 = contexts[0]["children"][0]
    assert frame_level_1["url"] == test_page_same_origin_frame

    assert len(frame_level_1["children"]) == 1
    frame_level_2 = frame_level_1["children"][0]
    assert frame_level_2["url"] == test_page

    await navigate_and_assert(bidi_session, frame_level_2, inline(PAGE_CONTENT))
