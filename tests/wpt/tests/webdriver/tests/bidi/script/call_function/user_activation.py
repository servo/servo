import pytest

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
@pytest.mark.parametrize("user_activation", [True, False])
async def test_userActivation(bidi_session, top_context, user_activation):
    # Consume any previously set activation.
    await bidi_session.script.evaluate(expression="""window.open();""",
                                       target=ContextTarget(
                                           top_context["context"]),
                                       await_promise=False)

    result = await bidi_session.script.call_function(
        function_declaration=
        "() => navigator.userActivation.isActive && navigator.userActivation.hasBeenActive",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        user_activation=user_activation)

    assert result == {"type": "boolean", "value": user_activation}


@pytest.mark.asyncio
@pytest.mark.parametrize("user_activation", [True, False])
async def test_userActivation_copy(bidi_session, top_context, user_activation):
    # Consume any previously set activation.
    await bidi_session.script.evaluate(expression="""window.open();""",
                                       target=ContextTarget(
                                           top_context["context"]),
                                       await_promise=False)

    result = await bidi_session.script.call_function(
        function_declaration=
        "() => document.body.appendChild(document.createTextNode('test')) && " +
        "document.execCommand('selectAll') && document.execCommand('copy')",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        user_activation=user_activation)

    assert result == {"type": "boolean", "value": user_activation}
