import os

import pytest

pytestmark = pytest.mark.asyncio


async def test_context(
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

    another_tab = await bidi_session.browsing_context.create(type_hint="tab")

    await bidi_session.browsing_context.navigate(
        context=another_tab["context"],
        url=inline(
            """
            <style>
            html, body { margin: 0; width: 100%; height: 100%; background: blue; }
            </style>
        """,
        ),
        wait="complete",
    )

    path = None
    try:
        result = await bidi_session.browsing_context.start_screencast(
            context=another_tab["context"]
        )
        path = result["path"]

        await produce_animation_frames(another_tab["context"])

        await bidi_session.browsing_context.stop_screencast(
            screencast=result["screencast"]
        )

        assert isinstance(result["path"], str)
        assert os.path.exists(result["path"])
        assert os.path.getsize(result["path"]) > 0

        # Verify that the right tab was recorded.
        color = await load_video_and_get_frame_color(result["path"])
        assert color["r"] == 0, color
        assert color["g"] == 0, color
        assert color["b"] == 251, color

    finally:
        if path is not None:
            os.remove(result["path"])
        await bidi_session.browsing_context.close(context=another_tab["context"])
