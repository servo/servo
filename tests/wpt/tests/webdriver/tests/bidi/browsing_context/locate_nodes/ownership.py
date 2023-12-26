import pytest

from ... import assert_handle


@pytest.mark.parametrize("ownership,has_handle", [
    ("root", True),
    ("none", False)
])
@pytest.mark.asyncio
async def test_root_ownership_of_located_nodes(bidi_session, inline, top_context, ownership, has_handle):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        ownership=ownership
    )

    assert len(result["nodes"]) == 1
    result_node = result["nodes"][0]

    assert_handle(result_node, has_handle)
