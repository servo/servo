import base64

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


async def test_rendering_isolation(
    bidi_session,
    current_session,
    top_context,
    new_tab,
    inline,
    compare_png_bidi,
):
    for context in [top_context, new_tab]:
        await bidi_session.browsing_context.navigate(
            context=context["context"], url=inline(PAGE), wait="complete"
        )

    await bidi_session.emulation.set_scrollbar_type_override(
        contexts=[new_tab["context"]],
        scrollbar_type="classic",
    )

    # Take Screenshot captures the current top-level browsing context.
    async def take_screenshot(context):
        current_session.window_handle = context["context"]
        return base64.b64decode(current_session.screenshot())

    data = await take_screenshot(new_tab)

    try:
        # Overriding the scrollbar type of another context doesn't change how
        # new_tab renders, whichever type that context had before.
        for scrollbar_type in ["classic", "overlay"]:
            await bidi_session.emulation.set_scrollbar_type_override(
                contexts=[top_context["context"]],
                scrollbar_type=scrollbar_type,
            )
            # Render top_context with its new type before new_tab is captured again.
            await take_screenshot(top_context)
            new_data = await take_screenshot(new_tab)
            comparison = await compare_png_bidi(new_data, data)
            assert comparison.equal()
    finally:
        current_session.window_handle = top_context["context"]
        await bidi_session.emulation.set_scrollbar_type_override(
            contexts=[top_context["context"]],
            scrollbar_type=None,
        )
