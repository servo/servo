import pytest

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("user_activation", [True, False])
async def test_userActivation(bidi_session, top_context, user_activation):
    await consume_activation(bidi_session, top_context["context"])

    result = await bidi_session.script.evaluate(
        expression=
        "navigator.userActivation.isActive && navigator.userActivation.hasBeenActive",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        user_activation=user_activation)

    assert result == {"type": "boolean", "value": user_activation}


@pytest.mark.parametrize("user_activation", [True, False])
async def test_userActivation_copy(bidi_session, top_context, user_activation):
    await consume_activation(bidi_session, top_context["context"])

    # Clipboard writes require document focus independently of user activation.
    await bidi_session.browsing_context.activate(
        context=top_context["context"])

    result = await bidi_session.script.evaluate(
        expression=
        "document.body.appendChild(document.createTextNode('test')) && document.execCommand('selectAll') && document.execCommand('copy')",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        user_activation=user_activation)

    assert result == {"type": "boolean", "value": user_activation}


async def test_userActivation_false_keeps_existing_activation(
    bidi_session, top_context
):
    await consume_activation(bidi_session, top_context["context"])

    await bidi_session.script.evaluate(
        expression="1",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True,
    )

    result = await bidi_session.script.evaluate(
        expression="navigator.userActivation.isActive",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=False,
    )

    assert result == {"type": "boolean", "value": True}


async def test_userActivation_true_after_consumed_activation(
    bidi_session, top_context
):
    await consume_activation(bidi_session, top_context["context"])

    # Activate and consume the activation in the same call.
    await bidi_session.script.evaluate(
        expression="window.open();",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True,
    )

    result = await bidi_session.script.evaluate(
        expression="navigator.userActivation.isActive",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        user_activation=True,
    )

    assert result == {"type": "boolean", "value": True}


async def consume_activation(bidi_session, context):
    # Consume any previously set activation.
    await bidi_session.script.evaluate(
        expression="window.open();",
        target=ContextTarget(context),
        await_promise=False,
    )
