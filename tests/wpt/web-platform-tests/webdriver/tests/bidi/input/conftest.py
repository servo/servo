import pytest

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
