import os

import pytest

pytestmark = pytest.mark.asyncio


async def test_stop_screencast(bidi_session, new_tab, inline, produce_animation_frames):
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

        stop_result = await bidi_session.browsing_context.stop_screencast(
            screencast=result["screencast"]
        )

        assert isinstance(stop_result["path"], str)
        assert os.path.exists(stop_result["path"])
        assert stop_result["path"] == result["path"]
        assert "error" not in stop_result
        assert os.path.getsize(stop_result["path"]) > 0
    finally:
        if path is not None:
            os.remove(path)
