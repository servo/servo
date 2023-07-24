import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest.fixture
def get_focused_key_input(bidi_session, top_context):
    """Get focused input element, containing pressed key data."""

    async def get_focused_key_input(context=top_context):
        return await bidi_session.script.call_function(
            function_declaration="""() => {
                const elem = document.getElementById("keys");
                elem.focus();
                return elem;
            }""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

    return get_focused_key_input


@pytest_asyncio.fixture(autouse=True)
async def release_actions(bidi_session, top_context):
    # release all actions after each test
    yield
    await bidi_session.input.release_actions(context=top_context["context"])


@pytest_asyncio.fixture
async def setup_key_test(load_static_test_page, get_focused_key_input):
    await load_static_test_page(page="test_actions.html")
    await get_focused_key_input()


@pytest_asyncio.fixture
async def setup_wheel_test(bidi_session, top_context, load_static_test_page):
    await load_static_test_page(page="test_actions_scroll.html")
    await bidi_session.script.evaluate(
        expression="document.scrollingElement.scrollTop = 0",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
