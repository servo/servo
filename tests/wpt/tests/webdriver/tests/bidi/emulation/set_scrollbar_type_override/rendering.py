import pytest

pytestmark = pytest.mark.asyncio

# A page that is taller than the viewport, so it has a vertical scrollbar, with
# a box at its right edge, which a classic scrollbar moves to the left.
PAGE = """
<style>
  body { height: 10000px; margin: 0; }
  div { position: fixed; top: 0; right: 0; width: 50px; height: 50px; background: blue; }
</style>
<div></div>
"""


async def test_rendering(bidi_session, new_tab, inline, compare_png_bidi):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline(PAGE), wait="complete"
    )

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )
    classic_data = await bidi_session.browsing_context.capture_screenshot(
        context=new_tab["context"]
    )

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="overlay",
    )
    overlay_data = await bidi_session.browsing_context.capture_screenshot(
        context=new_tab["context"]
    )

    comparison = await compare_png_bidi(classic_data, overlay_data)
    assert not comparison.equal()
