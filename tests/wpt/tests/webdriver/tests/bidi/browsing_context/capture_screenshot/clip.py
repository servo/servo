import pytest

import webdriver.bidi.error as error
from webdriver.bidi.modules.browsing_context import ElementOptions, BoxOptions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.image import png_dimensions


from . import (
    get_element_coordinates,
    get_physical_element_dimensions,
    get_reference_screenshot,
)
from ... import get_viewport_dimensions

pytestmark = pytest.mark.asyncio


async def test_clip_element(bidi_session, top_context, inline, compare_png_bidi):
    url = inline("<input />")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('input')",
        target=ContextTarget(top_context["context"]),
    )
    expected_size = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], clip=ElementOptions(element=element)
    )
    reference_data_dimensions = png_dimensions(reference_data)
    assert reference_data_dimensions == expected_size

    # Compare with the screenshot of the different element.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )
    data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], clip=ElementOptions(element=element)
    )

    assert png_dimensions(data) != reference_data_dimensions

    # Take a second screenshot that should be identical to validate that
    # we don't just always return false here.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"], clip=ElementOptions(element=element)
    )

    comparison = await compare_png_bidi(new_data, data)
    assert comparison.equal()


async def test_clip_box(bidi_session, top_context, inline, compare_png_bidi):
    url = inline("<input>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('input')",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    expected_size = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    reference_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        clip=BoxOptions(
            x=element_coordinates[0],
            y=element_coordinates[1],
            width=expected_size[0],
            height=expected_size[1],
        ),
    )
    reference_data_dimensions = png_dimensions(reference_data)
    assert reference_data_dimensions == expected_size

    # Compare with the screenshot of the different element.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    element_dimensions = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        clip=BoxOptions(
            x=element_coordinates[0],
            y=element_coordinates[1],
            width=element_dimensions[0],
            height=element_dimensions[1],
        ),
    )

    assert png_dimensions(data) != reference_data_dimensions

    # Take a second screenshot that should be identical to validate that
    # we don't just always return false here.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<div>foo</div>"), wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    element_dimensions = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        clip=BoxOptions(
            x=element_coordinates[0],
            y=element_coordinates[1],
            width=element_dimensions[0],
            height=element_dimensions[1],
        ),
    )

    comparison = await compare_png_bidi(new_data, data)
    assert comparison.equal()


async def test_clip_box_scroll_to(bidi_session, top_context, inline, compare_png_bidi):
    element_styles = "background-color: black; width: 50px; height:50px;"

    # Render an element inside of viewport for the reference.
    reference_data = await get_reference_screenshot(
        bidi_session,
        inline,
        top_context["context"],
        f"""<div style="{element_styles}"></div>""",
    )

    viewport_dimensions = await get_viewport_dimensions(bidi_session, top_context)

    # Render the same element outside of viewport.
    url = inline(
        f"""<div style="{element_styles} margin-top: {viewport_dimensions["height"]}px"></div>"""
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    element = await bidi_session.script.call_function(
        await_promise=False,
        function_declaration="""() => {{
            const element = document.querySelector('div');

            const rect = element.getBoundingClientRect();
            // Scroll to have the element in the viewport.
            window.scrollTo(0, rect.y);

            return element;
        }}""",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    element_dimensions = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        clip=BoxOptions(
            x=element_coordinates[0],
            y=element_coordinates[1],
            width=element_dimensions[0],
            height=element_dimensions[1],
        ),
    )

    assert png_dimensions(new_data) == element_dimensions

    comparison = await compare_png_bidi(reference_data, new_data)
    assert comparison.equal()


async def test_clip_box_partially_visible(
    bidi_session, top_context, inline, compare_png_bidi
):
    viewport_dimensions = await get_viewport_dimensions(bidi_session, top_context)
    element_styles = f"background-color: black; width: {viewport_dimensions['width']}px; height: 50px;"

    # Render an element fully inside of viewport for the reference.
    reference_data = await get_reference_screenshot(
        bidi_session,
        inline,
        top_context["context"],
        f"""<div style="{element_styles}"></div>""",
    )

    reference_data_dimensions = png_dimensions(reference_data)

    element_styles = f"background-color: black; width: {viewport_dimensions['width'] + 100}px; height: 50px;"

    url = inline(f"""<div style="{element_styles}"></div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    expected_size = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )
    new_data = await bidi_session.browsing_context.capture_screenshot(
        context=top_context["context"],
        clip=BoxOptions(
            x=element_coordinates[0],
            y=element_coordinates[1],
            width=expected_size[0],
            height=expected_size[1],
        ),
    )
    new_data_dimensions = png_dimensions(new_data)

    # Since the rendered element only partially visible,
    # the screenshot dimensions will not be equal the element size.
    assert new_data_dimensions != expected_size
    assert new_data_dimensions == reference_data_dimensions

    comparison = await compare_png_bidi(reference_data, new_data)
    assert comparison.equal()


@pytest.mark.parametrize("origin", ["document", "viewport"])
async def test_clip_box_outside_of_window_viewport(
    bidi_session, top_context, inline, compare_png_bidi, origin
):
    element_styles = "background-color: black; width: 50px; height:50px;"
    viewport_dimensions = await get_viewport_dimensions(bidi_session, top_context)

    # Render the element outside of viewport.
    url = inline(
        f"""<div style="{element_styles} margin-top: {viewport_dimensions["height"]}px"></div>"""
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    element = await bidi_session.script.call_function(
        await_promise=False,
        function_declaration="""() => document.querySelector('div')""",
        target=ContextTarget(top_context["context"]),
    )
    element_coordinates = await get_element_coordinates(
        bidi_session, top_context, element
    )
    element_dimensions = await get_physical_element_dimensions(
        bidi_session, top_context, element
    )

    if origin == "viewport":
        with pytest.raises(error.UnableToCaptureScreenException):
            await bidi_session.browsing_context.capture_screenshot(
                context=top_context["context"],
                clip=BoxOptions(
                    x=element_coordinates[0],
                    y=element_coordinates[1],
                    width=element_dimensions[0],
                    height=element_dimensions[1],
                ),
            )
    else:
        data = await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(
                x=element_coordinates[0],
                y=element_coordinates[1],
                width=element_dimensions[0],
                height=element_dimensions[1],
            ),
            origin="document",
        )
        assert png_dimensions(data) == element_dimensions

        # Render an element inside of viewport for the reference.
        reference_data = await get_reference_screenshot(
            bidi_session,
            inline,
            top_context["context"],
            f"""<div style="{element_styles}"></div>""",
        )

        comparison = await compare_png_bidi(reference_data, data)
        assert comparison.equal()


@pytest.mark.parametrize("origin", ["document", "viewport"])
async def test_clip_element_outside_of_window_viewport(
    bidi_session, top_context, inline, compare_png_bidi, origin
):
    viewport_dimensions = await get_viewport_dimensions(bidi_session, top_context)

    element_styles = "background-color: black; width: 50px; height:50px;"
    # Render element outside of viewport.
    url = inline(
        f"""<div style="{element_styles} margin-top: {viewport_dimensions["height"]}px"></div>"""
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    element = await bidi_session.script.evaluate(
        await_promise=False,
        expression="document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
    )

    if origin == "viewport":
        with pytest.raises(error.UnableToCaptureScreenException):
            await bidi_session.browsing_context.capture_screenshot(
                context=top_context["context"],
                clip=ElementOptions(element=element),
            )
    else:
        data = await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=ElementOptions(element=element),
            origin="document",
        )

        expected_size = await get_physical_element_dimensions(
            bidi_session, top_context, element
        )
        assert png_dimensions(data) == expected_size

        # Render an element inside of viewport for the reference.
        reference_data = await get_reference_screenshot(
            bidi_session,
            inline,
            top_context["context"],
            f"""<div style="{element_styles}"></div>""",
        )

        comparison = await compare_png_bidi(reference_data, data)
        assert comparison.equal()
