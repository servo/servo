import os

import pytest

pytestmark = pytest.mark.asyncio


async def test_start_screencast(
    bidi_session, new_tab, inline, produce_animation_frames
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    path = None
    try:
        result = await bidi_session.browsing_context.start_screencast(
            context=new_tab["context"]
        )
        path = result["path"]

        await produce_animation_frames(new_tab["context"])

        assert isinstance(result["path"], str)
        assert isinstance(result["screencast"], str)
        assert os.path.exists(result["path"])

        await bidi_session.browsing_context.stop_screencast(
            screencast=result["screencast"]
        )

        assert os.path.getsize(result["path"]) > 0
    finally:
        if path is not None:
            os.remove(path)


async def test_verify_file_content(
    bidi_session,
    new_tab,
    inline,
    produce_animation_frames,
    load_video_and_get_frame_color,
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(
            """
            <style>
              html, body { margin: 0; width: 100%; height: 100%; background: red; }
            </style>
        """,
        ),
        wait="complete",
    )

    path = None
    try:
        result = await bidi_session.browsing_context.start_screencast(
            context=new_tab["context"]
        )
        path = result["path"]

        await produce_animation_frames(new_tab["context"])

        await bidi_session.browsing_context.stop_screencast(
            screencast=result["screencast"]
        )

        assert isinstance(result["path"], str)
        assert os.path.exists(result["path"])
        assert os.path.getsize(result["path"]) > 0

        color = await load_video_and_get_frame_color(result["path"])

        assert color["r"] > 200, color
        assert color["g"] < 60, color
        assert color["b"] < 60, color
    finally:
        if path is not None:
            os.remove(result["path"])
