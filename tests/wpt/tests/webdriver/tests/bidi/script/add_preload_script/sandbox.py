import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare

pytestmark = pytest.mark.asyncio


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


async def test_arguments(
    bidi_session,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    add_preload_script,
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    await add_preload_script(
        function_declaration="(channel) => channel('foo')",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        sandbox="sandbox",
    )

    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    event_data = await wait_for_future_safe(on_script_message)

    result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="1",
        target=ContextTarget(new_tab["context"], "sandbox"),
        await_promise=True,
    )

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "foo"},
            "source": {
                "realm": result["realm"],
                "context": new_tab["context"],
            },
        },
        event_data,
    )

