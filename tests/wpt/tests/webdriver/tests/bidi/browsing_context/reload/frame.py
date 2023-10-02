import pytest

from . import reload_and_assert

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_origin(bidi_session, new_tab, inline, domain):
    frame_url = inline("frame")
    parent_url = inline(f"<iframe src='{frame_url}'></iframe>", domain=domain)

    # Navigate and assert (top-level).
    result = await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=parent_url, wait="complete")
    assert result["url"] == parent_url

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab['context'])
    assert len(contexts) == 1
    assert contexts[0]["url"] == parent_url

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_url

    # Reload and assert (frame).
    reload_and_assert(bidi_session, frame, last_navigation=result["navigation"], url=frame_url)
