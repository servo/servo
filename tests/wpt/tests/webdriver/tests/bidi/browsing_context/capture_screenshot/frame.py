import pytest

from tests.support.image import png_dimensions
from tests.support.screenshot import (DEFAULT_CONTENT,
                                      REFERENCE_CONTENT,
                                      REFERENCE_STYLE,
                                      OUTER_IFRAME_STYLE,
                                      INNER_IFRAME_STYLE)

from . import get_physical_viewport_dimensions


@pytest.mark.asyncio
async def test_iframe(bidi_session, top_context, inline, iframe):
    viewport_size = await get_physical_viewport_dimensions(bidi_session, top_context)

    iframe_content = f"{INNER_IFRAME_STYLE}{DEFAULT_CONTENT}"
    url = inline(f"{OUTER_IFRAME_STYLE}{iframe(iframe_content)}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=url,
                                                 wait="complete")
    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])
    assert png_dimensions(reference_data) == viewport_size

    all_contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    frame_context = all_contexts[0]["children"][0]

    data = await bidi_session.browsing_context.capture_screenshot(context=frame_context["context"])

    assert png_dimensions(data) < png_dimensions(reference_data)


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.asyncio
async def test_context_origin(bidi_session, top_context, inline, iframe, compare_png_bidi, domain):
    expected_size = await get_physical_viewport_dimensions(bidi_session, top_context)

    initial_url = inline(f"{REFERENCE_STYLE}{REFERENCE_CONTENT}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=initial_url,
                                                 wait="complete")

    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"])
    assert png_dimensions(reference_data) == expected_size

    iframe_content = f"{INNER_IFRAME_STYLE}{DEFAULT_CONTENT}"
    new_url = inline(f"{OUTER_IFRAME_STYLE}{iframe(iframe_content, domain=domain)}")
    await bidi_session.browsing_context.navigate(context=top_context["context"],
                                                 url=new_url,
                                                 wait="complete")

    data = await bidi_session.browsing_context.capture_screenshot(context=top_context["context"])
    comparison = await compare_png_bidi(data, reference_data)

    assert comparison.equal()
