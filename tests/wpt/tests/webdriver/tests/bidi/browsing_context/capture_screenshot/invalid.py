import pytest

from ... import get_viewport_dimensions

import webdriver.bidi.error as error
from webdriver.bidi.modules.browsing_context import ElementOptions, BoxOptions
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(context=value)


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_invalid_frame(bidi_session, value):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.capture_screenshot(context=value)


async def test_closed_frame(bidi_session, top_context, inline, add_and_remove_iframe):
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    frame_id = await add_and_remove_iframe(top_context)
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.capture_screenshot(context=frame_id)


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_clip_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"], clip=value
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_clip_type_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"], clip={"type": value}
        )


async def test_params_clip_type_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"], clip={"type": "foo"}
        )


@pytest.mark.parametrize("value", [None, False, 42, "foo", []])
async def test_params_clip_element_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=ElementOptions(element=value),
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_clip_element_sharedId_invalid_type(
    bidi_session, top_context, value
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=ElementOptions(element={"shareId": value}),
        )


async def test_params_clip_element_sharedId_invalid_value(bidi_session, top_context):
    with pytest.raises(error.NoSuchNodeException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=ElementOptions(element={"sharedId": "foo"}),
        )


@pytest.mark.parametrize("value", [None, False, "foo", {}, []])
async def test_params_clip_viewport_x_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(x=value, y=0, width=0, height=0),
        )


@pytest.mark.parametrize("value", [None, False, "foo", {}, []])
async def test_params_clip_viewport_y_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(x=0, y=value, width=0, height=0),
        )


@pytest.mark.parametrize("value", [None, False, "foo", {}, []])
async def test_params_clip_viewport_width_invalid_type(
    bidi_session, top_context, value
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(x=0, y=0, width=value, height=0),
        )


@pytest.mark.parametrize("value", [None, False, "foo", {}, []])
async def test_params_clip_viewport_height_invalid_type(
    bidi_session, top_context, value
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(x=0, y=0, width=0, height=value),
        )


async def test_params_clip_viewport_dimensions_invalid_value(bidi_session, top_context):
    with pytest.raises(error.UnableToCaptureScreenException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(x=0, y=0, width=0, height=0),
        )


async def test_params_clip_viewport_outside_of_window_viewport(
    bidi_session, top_context
):
    viewport_dimensions = await get_viewport_dimensions(bidi_session, top_context)

    with pytest.raises(error.UnableToCaptureScreenException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=BoxOptions(
                x=viewport_dimensions["width"],
                y=viewport_dimensions["height"],
                width=1,
                height=1,
            ),
        )


async def test_params_clip_element_outside_of_window_viewport(
    bidi_session, top_context, inline
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

    with pytest.raises(error.UnableToCaptureScreenException):
        await bidi_session.browsing_context.capture_screenshot(
            context=top_context["context"],
            clip=ElementOptions(element=element),
        )
