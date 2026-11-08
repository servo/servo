import pytest

from tests.support.image import png_dimensions
from tests.support.screenshot import (DEFAULT_CONTENT,
                                      REFERENCE_CONTENT,
                                      REFERENCE_STYLE,
                                      OUTER_IFRAME_STYLE,
                                      INNER_IFRAME_STYLE)

from . import capture_screenshot_and_assert_visual_viewport_dimensions

pytestmark = pytest.mark.asyncio


# The 200px fixture does not create viewport overflow.
async def test_iframe(bidi_session, top_context, inline, iframe):
    iframe_content = f"{INNER_IFRAME_STYLE}{DEFAULT_CONTENT}"
    url = inline(f"{OUTER_IFRAME_STYLE}{iframe(iframe_content)}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url,
                                                 wait="complete")
    reference_data = await capture_screenshot_and_assert_visual_viewport_dimensions(
        bidi_session, top_context)

    all_contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context = all_contexts[0]["children"][0]

    data = await bidi_session.browsing_context.capture_screenshot(context=frame_context["context"])

    assert png_dimensions(data) < png_dimensions(reference_data)


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_context_origin(bidi_session, top_context, inline, iframe, compare_png_bidi, domain):
    initial_url = inline(f"{REFERENCE_STYLE}{REFERENCE_CONTENT}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=initial_url,
                                                 wait="complete")

    reference_data = await capture_screenshot_and_assert_visual_viewport_dimensions(
        bidi_session, top_context)

    iframe_content = f"{INNER_IFRAME_STYLE}{DEFAULT_CONTENT}"
    new_url = inline(f"{OUTER_IFRAME_STYLE}{iframe(iframe_content, domain=domain)}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=new_url,
                                                 wait="complete")

    data = await capture_screenshot_and_assert_visual_viewport_dimensions(
        bidi_session, top_context)
    comparison = await compare_png_bidi(data, reference_data)

    assert comparison.equal()
