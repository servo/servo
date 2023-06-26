import pytest

import webdriver.bidi.error as error


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
@pytest.mark.asyncio
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.capture_screenshot(context=value)


@pytest.mark.asyncio
async def test_invalid_frame(bidi_session, top_context, inline):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.capture_screenshot(context="_invalid_")


@pytest.mark.asyncio
async def test_closed_frame(bidi_session, top_context, inline, add_and_remove_iframe):
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
    frame_id = await add_and_remove_iframe(top_context)
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.capture_screenshot(context=frame_id)
