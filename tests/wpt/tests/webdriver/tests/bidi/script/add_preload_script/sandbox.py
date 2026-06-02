import pytest

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
async def test_add_preload_script_to_sandbox(bidi_session, add_preload_script):
    # Add preload script to make changes in window
    await add_preload_script(function_declaration="() => { window.foo = 1; }")
    # Add preload script to make changes in sandbox
    await add_preload_script(
        function_declaration="() => { window.bar = 2; }", sandbox="sandbox"
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")

    # Check that changes from the first preload script are not present in sandbox
    result_in_sandbox = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab["context"], "sandbox"),
        await_promise=True,
    )
    assert result_in_sandbox == {"type": "undefined"}

    # Make sure that changes from the second preload script are not present in window
    result = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}

    # Make sure that changes from the second preload script are present in sandbox
    result_in_sandbox = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(new_tab["context"], "sandbox"),
        await_promise=True,
    )
    assert result_in_sandbox == {"type": "number", "value": 2}


@pytest.mark.asyncio
async def test_remove_properties_set_by_preload_script(
    bidi_session, add_preload_script, new_tab, inline
):
    await add_preload_script(function_declaration="() => { window.foo = 42 }")
    await add_preload_script(function_declaration="() => { window.foo = 50 }", sandbox="sandbox_1")

    url = inline("<script>delete window.foo</script>")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete",
    )

    # Check that page script could access a function set up by the preload script
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result == {"type": "undefined"}

    # Check that page script could access a function set up by the preload script
    result = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab["context"], sandbox="sandbox_1"),
        await_promise=True,
    )
    assert result == {"type": "number", "value": 50}
