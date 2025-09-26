import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def is_scripting_enabled(bidi_session):
    """A fixture that returns a function to check if scripting is enabled in a
    given browsing context."""

    async def is_scripting_enabled(context):
        result = await bidi_session.script.evaluate(
            # `noscript` elements do not render their children if scripting is
            # enabled.
            expression="""(()=>{
                const n = document.createElement('noscript');
                n.innerHTML = '<div></div>';
                return n.childElementCount === 0;
            })()""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        return result["value"]

    return is_scripting_enabled
