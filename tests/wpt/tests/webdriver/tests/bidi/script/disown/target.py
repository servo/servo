import pytest

import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget, RealmTarget

from ... import assert_handle

pytestmark = pytest.mark.asyncio


async def test_realm(bidi_session, top_context, call_function):
    remote_value = await bidi_session.script.evaluate(
        raw_result=True,
        expression="({a:1})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    assert_handle(remote_value["result"], True)

    result = await call_function("arg => arg.a", [remote_value["result"]])

    assert result == {"type": "number", "value": 1}

    await bidi_session.script.disown(
        handles=[remote_value["result"]["handle"]],
        target=RealmTarget(remote_value["realm"]),
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value["result"]])


async def test_sandbox(bidi_session, top_context, call_function):
    # Create a remote value outside of any sandbox
    remote_value = await bidi_session.script.evaluate(
        expression="({a:'without sandbox'})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    # Create a remote value from a sandbox
    sandbox_value = await bidi_session.script.evaluate(
        expression="({a:'with sandbox'})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"], "basic_sandbox"),
    )

    # Try to disown the non-sandboxed remote value from the sandbox
    await bidi_session.script.disown(
        handles=[remote_value["handle"]],
        target=ContextTarget(top_context["context"], "basic_sandbox"),
    )

    # Check that the remote value is still working
    result = await call_function("arg => arg.a", [remote_value])
    assert result == {"type": "string", "value": "without sandbox"}

    # Try to disown the sandbox value:
    # - from the non-sandboxed top context
    # - from another sandbox
    await bidi_session.script.disown(
        handles=[sandbox_value["handle"]], target=ContextTarget(top_context["context"])
    )
    await bidi_session.script.disown(
        handles=[sandbox_value["handle"]],
        target=ContextTarget(top_context["context"], "another_sandbox"),
    )

    # Check that the sandbox remote value is still working
    result = await call_function(
        "arg => arg.a", [sandbox_value], sandbox="basic_sandbox"
    )
    assert result == {"type": "string", "value": "with sandbox"}

    # Disown the sandbox remote value from the correct sandbox
    await bidi_session.script.disown(
        handles=[sandbox_value["handle"]],
        target=ContextTarget(top_context["context"], "basic_sandbox"),
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [sandbox_value], sandbox="basic_sandbox")

    # Disown the non-sandboxed remote value from the top context
    await bidi_session.script.disown(
        handles=[remote_value["handle"]], target=ContextTarget(top_context["context"])
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value], sandbox="basic_sandbox")


async def test_context_and_realm(bidi_session, top_context, new_tab, call_function):
    # Create a remote value outside of any sandbox.
    result_in_default_realm = await bidi_session.script.evaluate(
        raw_result=True,
        expression="({a:'without sandbox'})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(new_tab["context"]),
    )
    remote_value = result_in_default_realm["result"]

    # Create a remote value from a sandbox.
    result_in_sandbox = await bidi_session.script.evaluate(
        raw_result=True,
        expression="({a:'with sandbox'})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"], "basic_sandbox"),
    )
    sandbox_value = result_in_sandbox["result"]

    # Make sure that realm argument is ignored and the value is disowned
    # in the default realm of another context.
    await bidi_session.script.disown(
        handles=[remote_value["handle"]],
        target={
            "context": new_tab["context"],
            "realm": result_in_default_realm["realm"]
        }
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value], None, new_tab["context"])

    # Check that the sandbox remote value is still working.
    result = await call_function(
        "arg => arg.a", [sandbox_value], sandbox="basic_sandbox"
    )
    assert result == {"type": "string", "value": "with sandbox"}
