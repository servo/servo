import os
from urllib.parse import quote

import pytest
from webdriver.bidi.modules.browser import ClientWindowRectState
from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict

pytestmark = pytest.mark.asyncio


async def get_video_dimensions(bidi_session, context, path, url, inline):
    await bidi_session.browsing_context.navigate(
        context=context, url=inline("<div>foo</div>"), wait="complete"
    )

    video_url = url(
        "/webdriver/tests/support/http_handlers/serve_video.py",
        query=f"path={quote(path)}",
    )

    result = await bidi_session.script.call_function(
        function_declaration="""async (videoUrl) => {
            const video = document.createElement("video");
            video.preload = "metadata";
            document.body.appendChild(video);
            await new Promise((resolve, reject) => {
                video.addEventListener("loadedmetadata", resolve, { once: true });
                video.addEventListener("error", () => {
                    const err = video.error;
                    reject(new Error(
                        "video failed to load: " +
                        (err ? `code=${err.code} message=${err.message}` : "unknown")
                    ));
                }, { once: true });
                video.src = videoUrl;
            });

            const dimensions = { width: video.videoWidth, height: video.videoHeight };

            video.remove();
            return dimensions;
        }""",
        arguments=[{"type": "string", "value": video_url}],
        target=ContextTarget(context),
        await_promise=True,
    )

    return remote_mapping_to_dict(result["value"])


@pytest.mark.parametrize(
    "expected_video_dimensions",
    [
        ({"width": 200}),
        ({"height": 300}),
        ({"width": 350, "height": 175}),
    ],
)
async def test_video_dimensions(
    bidi_session,
    top_context,
    new_tab,
    inline,
    url,
    produce_animation_frames,
    expected_video_dimensions,
):
    # Set viewport to have an expected viewport ratio for video.
    await bidi_session.browsing_context.set_viewport(
        context=new_tab["context"],
        viewport={"width": 600, "height": 300}
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<div>foo</div>"),
        wait="none",
    )

    path = None
    try:
        result = await bidi_session.browsing_context.start_screencast(
            context=new_tab["context"], video=expected_video_dimensions
        )
        path = result["path"]

        await produce_animation_frames(new_tab["context"])

        await bidi_session.browsing_context.stop_screencast(
            screencast=result["screencast"],
        )

        assert os.path.getsize(result["path"]) > 0

        dimensions = await get_video_dimensions(
            bidi_session, top_context["context"], result["path"], url, inline
        )

        for key in expected_video_dimensions:
            assert dimensions[key] == expected_video_dimensions[key]
    finally:
        if path is not None:
            os.remove(path)
