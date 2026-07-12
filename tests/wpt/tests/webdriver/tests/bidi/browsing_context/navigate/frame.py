import pytest
from webdriver.bidi.modules.script import ContextTarget

from .. import navigate_and_assert

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


async def test_subframe_replacestate(bidi_session, new_tab, inline):
    script_url = "/webdriver/tests/bidi/browsing_context/support/empty.js"
    # An iframe that calls history.replaceState on itself, must
    # not cause the top-level navigation to complete prematurely.
    # The slow parser-blocking script delays the page load so that premature
    # completion is detectable via document.readyState.
    iframe_url = inline('<script>history.replaceState(null, "", "#hash");</script>')
    url = inline(
        f'<iframe src="{iframe_url}"></iframe>'
        f'<script src="{script_url}?pipe=trickle(d1)"></script>'
    )

    await navigate_and_assert(bidi_session, new_tab, url)

    ready_state = await bidi_session.script.evaluate(
        expression="document.readyState",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    assert ready_state["value"] == "complete"


async def test_subframe_error_page(bidi_session, new_tab, inline):
    # An iframe served with X-Frame-Options: DENY will fail to load and show
    # an error page. This must not cause the top-level navigation to fail.
    iframe_url = inline(
        PAGE_CONTENT,
        parameters={"pipe": "header(X-Frame-Options, DENY)"},
    )
    url = inline(f'<iframe src="{iframe_url}"></iframe>')

    await navigate_and_assert(bidi_session, new_tab, url)

    ready_state = await bidi_session.script.evaluate(
        expression="document.readyState",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    assert ready_state["value"] == "complete"
