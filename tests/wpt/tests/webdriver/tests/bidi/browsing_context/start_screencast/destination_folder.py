import os
from pathlib import Path

import pytest

pytestmark = pytest.mark.asyncio


async def test_destination_folder(
    bidi_session, new_tab, inline, produce_animation_frames, tmp_path
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div>foo</div>"),
        wait="complete",
    )

    result = await bidi_session.browsing_context.start_screencast(
        context=new_tab["context"], destination_folder=str(tmp_path)
    )

    await produce_animation_frames(new_tab["context"])

    assert isinstance(result["path"], str)
    assert os.path.exists(result["path"])
    assert Path(result["path"]).parent.resolve() == tmp_path.resolve()

    await bidi_session.browsing_context.stop_screencast(
        screencast=result["screencast"]
    )

    assert os.path.getsize(result["path"]) > 0
