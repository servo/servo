import pytest

from webdriver.bidi.modules.script import ContextTarget


@pytest.mark.asyncio
async def test_remove_preload_script_from_sandbox(bidi_session):
    # Add preload script to make changes in window
    script_1 = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.foo = 1; }",
    )
    # Add preload script to make changes in sandbox
    script_2 = await bidi_session.script.add_preload_script(
        function_declaration="() => { window.bar = 2; }", sandbox="sandbox"
    )

    # Remove first preload script
    await bidi_session.script.remove_preload_script(
        script=script_1,
    )
    # Remove second preload script
    await bidi_session.script.remove_preload_script(
        script=script_2,
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")

    # Make sure that changes from first preload script were not applied
    result_in_window = await bidi_session.script.evaluate(
        expression="window.foo",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
    )
    assert result_in_window == {"type": "undefined"}

    # Make sure that changes from second preload script were not applied
    result_in_sandbox = await bidi_session.script.evaluate(
        expression="window.bar",
        target=ContextTarget(new_tab["context"], "sandbox"),
        await_promise=True,
    )
    assert result_in_sandbox == {"type": "undefined"}
