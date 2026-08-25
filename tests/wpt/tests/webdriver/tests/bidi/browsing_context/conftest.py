import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def produce_animation_frames(bidi_session):
    async def produce_animation_frames(context, count=100):
        await bidi_session.script.call_function(
            function_declaration="""async (times) => {
                for (let i=0; i<times; i++) {
                    await new Promise(f => window.requestAnimationFrame(() => window.requestAnimationFrame(f)));
                }
            }""",
            arguments=[{"type": "number", "value": count}],
            target=ContextTarget(context),
            await_promise=True,
        )

    return produce_animation_frames
