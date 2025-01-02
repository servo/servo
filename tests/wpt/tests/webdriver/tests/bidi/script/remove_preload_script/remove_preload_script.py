import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_remove_preload_script(bidi_session, type_hint):
    script = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.foo='bar'; }"
    )

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_context["context"]),
        await_promise=True,
    )
    assert result == {"type": "string", "value": "bar"}

    await bidi_session.script.remove_preload_script(script=script)

    new_tab_2 = await bidi_session.browsing_context.create(type_hint=type_hint)

    # Check that changes from preload script were not applied after script was removed
    result_2 = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab_2["context"]),
        await_promise=True,
    )
    assert result_2 == {"type": "undefined"}


@pytest.mark.asyncio
async def test_remove_preload_script_twice(bidi_session):
    script = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.foo='bar'; }"
    )

    await bidi_session.script.remove_preload_script(script=script)

    # Check that we can not remove the same script twice
    with pytest.raises(error.NoSuchScriptException):
        await bidi_session.script.remove_preload_script(script=script)


@pytest.mark.asyncio
async def test_remove_one_of_preload_scripts(bidi_session):
    script_1 = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.bar='foo'; }"
    )
    script_2 = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.baz='bar'; }"
    )

    # Remove one of the scripts
    await bidi_session.script.remove_preload_script(script=script_1)

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")

    # Check that the first script didn't run
    result = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}

    # Check that the second script still applied the changes to the window
    result_2 = await bidi_session.script.evaluate(
        expression="window.baz",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result_2 == {"type": "string", "value": "bar"}

    # Clean up the second script
    await bidi_session.script.remove_preload_script(script=script_2)


@pytest.mark.asyncio
async def test_remove_script_set_up_for_one_context(
    bidi_session, add_preload_script, new_tab, test_page, test_page_cross_origin
):
    script = await add_preload_script(
        function_declaration="() => { window.baz = 42; }",
        contexts=[new_tab["context"]],
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_page,
        wait="complete",
    )

    # Check that preload script applied the changes to the context
    result = await bidi_session.script.evaluate(
        expression="window.baz",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "number", "value": 42}

    await bidi_session.script.remove_preload_script(script=script)

    # Navigate again to see that preload script didn't run
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_page_cross_origin,
        wait="complete",
    )

    result = await bidi_session.script.evaluate(
        expression="window.baz",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}
