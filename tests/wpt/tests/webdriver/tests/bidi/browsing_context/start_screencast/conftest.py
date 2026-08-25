from pathlib import Path

import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict


@pytest_asyncio.fixture
async def load_video_and_get_frame_color(bidi_session, top_context):
    async def load_video_and_get_frame_color(path):
        video_url = Path(path).as_uri()

        # We have to navigate to the page with "file" protocol
        # to be able to load the video file in the video tag.
        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=video_url,
            wait="complete",
        )

        sample = await bidi_session.script.call_function(
            function_declaration="""async (videoUrl) => {
                const video = document.querySelector("video");
                await new Promise((resolve, reject) => {
                    video.addEventListener("loadeddata", resolve, { once: true });
                    video.addEventListener("error", () => {
                        const err = video.error;
                        reject(new Error(
                            "video failed to load: " +
                            (err ? `code=${err.code} message=${err.message}` : "unknown")
                        ));
                    }, { once: true });
                    video.src = videoUrl;
                });
                const canvas = document.createElement("canvas");
                canvas.width = video.videoWidth;
                canvas.height = video.videoHeight;
                const ctx = canvas.getContext("2d");
                ctx.drawImage(video, 0, 0);

                // Pick a pixel from the center of the video.
                const data = ctx.getImageData(
                    Math.floor(video.videoWidth / 2),
                    Math.floor(video.videoHeight / 2),
                    1, 1
                ).data;

                video.remove();
                canvas.remove();

                return { r: data[0], g: data[1], b: data[2] };
            }""",
            arguments=[{"type": "string", "value": video_url}],
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )

        return remote_mapping_to_dict(sample["value"])

    return load_video_and_get_frame_color
