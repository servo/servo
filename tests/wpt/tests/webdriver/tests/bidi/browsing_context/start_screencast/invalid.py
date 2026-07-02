import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(context=value)


async def test_params_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.start_screencast(context="_invalid_")


async def test_params_context_non_top_level(
    bidi_session, top_context, test_page_same_origin_frame
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page_same_origin_frame,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=frames[0]["context"]
        )


@pytest.mark.parametrize("value", get_invalid_cases("string", nullable=True))
async def test_params_mime_type_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], mime_type=value
        )


async def test_params_mime_type_invalid_value(bidi_session, top_context):
    with pytest.raises(error.UnsupportedOperationException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], mime_type="video/unsupported"
        )


@pytest.mark.parametrize("value", get_invalid_cases("dict", nullable=True))
async def test_params_video_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video=value
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_params_video_frame_rate_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"frameRate": value}
        )


@pytest.mark.parametrize("value", [-1, 1.1])
async def test_params_video_frame_rate_invalid_value(
    bidi_session, top_context, value
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"frameRate": value}
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_params_video_height_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"height": value}
        )


@pytest.mark.parametrize("value", [-1, 1.1])
async def test_params_video_height_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"height": value}
        )


@pytest.mark.parametrize("value", get_invalid_cases("number"))
async def test_params_video_width_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"width": value}
        )


@pytest.mark.parametrize("value", [-1, 1.1])
async def test_params_video_width_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], video={"width": value}
        )


@pytest.mark.parametrize("value", get_invalid_cases("boolean", nullable=True))
async def test_params_audio_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.start_screencast(
            context=top_context["context"], audio=value
        )
