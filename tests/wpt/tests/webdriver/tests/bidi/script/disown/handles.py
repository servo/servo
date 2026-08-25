import pytest

import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget

from ... import assert_handle

pytestmark = pytest.mark.asyncio


async def test_basic_handle(bidi_session, top_context, call_function):
    remote_value = await bidi_session.script.evaluate(
        expression="({a:1})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    assert_handle(remote_value, True)

    result = await call_function("arg => arg.a", [remote_value])

    assert result == {"type": "number", "value": 1}

    await bidi_session.script.disown(
        handles=[remote_value["handle"]], target=ContextTarget(top_context["context"])
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value])


async def test_multiple_handles_for_different_objects(
    bidi_session, top_context, call_function
):
    # Create a handle
    remote_value_a = await bidi_session.script.evaluate(
        expression="({a:1})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    remote_value_b = await bidi_session.script.evaluate(
        expression="({b:2})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    remote_value_c = await bidi_session.script.evaluate(
        expression="({c:3})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    assert_handle(remote_value_a, True)
    assert_handle(remote_value_b, True)
    assert_handle(remote_value_c, True)

    # disown a and b
    await bidi_session.script.disown(
        handles=[remote_value_a["handle"], remote_value_b["handle"]],
        target=ContextTarget(top_context["context"]),
    )

    # using handle a or b should raise an exception
    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value_a])

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.b", [remote_value_b])

    # remote value c should still work
    result = await call_function("arg => arg.c", [remote_value_c])

    assert result == {"type": "number", "value": 3}

    # disown c
    await bidi_session.script.disown(
        handles=[remote_value_c["handle"]], target=ContextTarget(top_context["context"])
    )

    # using handle c should raise an exception
    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.c", [remote_value_c])


async def test_multiple_handles_for_same_object(
    bidi_session, top_context, call_function
):
    remote_value1 = await bidi_session.script.evaluate(
        expression="window.test = { a: 1 }; window.test",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )
    assert_handle(remote_value1, True)

    remote_value2 = await bidi_session.script.evaluate(
        expression="window.test",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )
    assert_handle(remote_value2, True)

    # Check that both handles can be used
    result = await call_function("arg => arg.a", [remote_value1])
    assert result == {"type": "number", "value": 1}

    result = await call_function("arg => arg.a", [remote_value2])
    assert result == {"type": "number", "value": 1}

    # Check that both handles point to the same value
    result = await call_function(
        "(arg1, arg2) => arg1 === arg2", [remote_value1, remote_value2]
    )
    assert result == {"type": "boolean", "value": True}

    # Disown the handle 1
    await bidi_session.script.disown(
        handles=[remote_value1["handle"]], target=ContextTarget(top_context["context"])
    )

    # Using handle 1 should raise an exception
    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value1])

    # Using handle 2 should still work
    result = await call_function("arg => arg.a", [remote_value2])
    assert result == {"type": "number", "value": 1}

    # Disown the handle 2
    await bidi_session.script.disown(
        handles=[remote_value2["handle"]], target=ContextTarget(top_context["context"])
    )

    # Using handle 2 should raise an exception
    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value2])


async def test_unknown_handle(bidi_session, top_context, call_function):
    # Create a handle
    remote_value = await bidi_session.script.evaluate(
        expression="({a:1})",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    assert_handle(remote_value, True)

    # An unknown handle should not remove other handles, and should not fail
    await bidi_session.script.disown(
        handles=["unknown_handle"], target=ContextTarget(top_context["context"])
    )

    result = await call_function("arg => arg.a", [remote_value])

    assert result == {"type": "number", "value": 1}

    # Passing an unknown handle with an existing handle should disown the existing one
    await bidi_session.script.disown(
        handles=["unknown_handle", remote_value["handle"]],
        target=ContextTarget(top_context["context"]),
    )

    with pytest.raises(error.NoSuchHandleException):
        await call_function("arg => arg.a", [remote_value])
