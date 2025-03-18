import pytest

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_add_preload_script_to_user_context(
    bidi_session, add_preload_script, create_user_context, inline
):
    user_context = await create_user_context()
    await add_preload_script(
        function_declaration="() => { window.foo='bar'; }", user_contexts=[user_context]
    )

    new_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Check that preload script applied the changes to the tab in the user context.
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_1["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "bar"}

    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_context_1["context"],
        url=url,
        wait="complete",
    )

    # Check that preload script was applied after navigation
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_1["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "bar"}

    # Create a new browsing context in the default user context.
    new_context_2 = await bidi_session.browsing_context.create(type_hint="tab")

    # Check that preload script didn't apply the changes to the tab in the default user context.
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_2["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}

    await bidi_session.browsing_context.close(context=new_context_1["context"])
    await bidi_session.browsing_context.close(context=new_context_2["context"])


async def test_add_preload_script_to_default_user_context(
    bidi_session, add_preload_script, inline, create_user_context
):
    await add_preload_script(
        function_declaration="() => { window.foo='bar'; }", user_contexts=["default"]
    )

    new_context_1 = await bidi_session.browsing_context.create(type_hint="tab")

    # Check that preload script applied the changes to the tab in the user context.
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_1["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "bar"}

    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=new_context_1["context"],
        url=url,
        wait="complete",
    )

    # Check that preload script was applied after navigation
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_1["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "bar"}

    user_context = await create_user_context()
    # Create a new browsing context in the other user context.
    new_context_2 = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )

    # Check that preload script didn't apply the changes to the tab in the default user context.
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_2["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}

    await bidi_session.browsing_context.close(context=new_context_1["context"])
    await bidi_session.browsing_context.close(context=new_context_2["context"])


async def test_add_preload_script_to_multiple_user_contexts(
    bidi_session, add_preload_script, create_user_context
):
    user_context_1 = await create_user_context()
    user_context_2 = await create_user_context()

    await add_preload_script(
        function_declaration="() => { window.foo='bar'; }",
        user_contexts=[user_context_1, user_context_2],
    )

    new_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    new_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )

    # Check that preload script applied the changes to the tabs in the both user contexts.
    result_1 = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_1["context"]),
        await_promise=True,
    )
    assert result_1 == {"type": "string", "value": "bar"}

    result_2 = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context_2["context"]),
        await_promise=True,
    )
    assert result_2 == {"type": "string", "value": "bar"}

    await bidi_session.browsing_context.close(context=new_context_1["context"])
    await bidi_session.browsing_context.close(context=new_context_2["context"])


async def test_identical_user_contexts(
    bidi_session, add_preload_script, create_user_context
):
    user_context = await create_user_context()
    await add_preload_script(
        function_declaration="() => { window.foo = window.foo ? window.foo + 1 : 1; }",
        user_contexts=[user_context, user_context],
    )

    new_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Check that preload script applied the changes only once
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context["context"]),
        await_promise=True,
    )
    assert result == {"type": "number", "value": 1}

    await bidi_session.browsing_context.close(context=new_context["context"])
