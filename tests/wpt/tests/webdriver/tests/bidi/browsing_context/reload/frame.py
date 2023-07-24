import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_origin(bidi_session, new_tab, inline, domain):
    frame_start_url = inline("frame")
    url_before = inline(f"<iframe src='{frame_start_url}'></iframe>",
                        domain=domain)

    # Navigate and assert (top-level).
    result = await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=url_before, wait="complete")
    assert result["url"] == url_before

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab['context'])
    assert len(contexts) == 1
    assert contexts[0]["url"] == url_before

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]
    assert frame["url"] == frame_start_url

    # Reload and assert (frame).
    result = await bidi_session.browsing_context.reload(
        context=frame['context'], wait="complete")
    assert result == {}

    contexts = await bidi_session.browsing_context.get_tree(
        root=frame['context'])
    assert len(contexts) == 1
    assert contexts[0]["url"] == frame_start_url
