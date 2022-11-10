import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare


@pytest.mark.asyncio
async def test_this(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="function(){return this.some_property}",
        this={
            "type": "object",
            "value": [[
                "some_property",
                {
                    "type": "number",
                    "value": 42
                }]]},
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    assert result == {
        'type': 'number',
        'value': 42}


@pytest.mark.asyncio
async def test_default_this(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="function(){return this}",
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    recursive_compare({
        "type": 'window',
    }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "value_fn, function_declaration",
    [
        (
            lambda value: value,
            "function() { return this === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "object", "value": [["nested", value]]}),
            "function() { return this.nested === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "array", "value": [value]}),
            "function() { return this[0] === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "map", "value": [["foobar", value]]}),
            "function() { return this.get('foobar') === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "set", "value": [value]}),
            "function() { return this.has(window.SOME_OBJECT); }",
        ),
    ],
)
async def test_remote_value_deserialization(
    bidi_session, top_context, call_function, evaluate, value_fn, function_declaration
):
    remote_value = await evaluate(
        "window.SOME_OBJECT = {SOME_PROPERTY:'SOME_VALUE'}; window.SOME_OBJECT",
        result_ownership="root",
    )

    # Check that a remote value can be successfully deserialized as the "this"
    # parameter and compared against the original object in the page.
    result = await call_function(
        function_declaration=function_declaration,
        this=value_fn(remote_value),
    )
    assert result == {"type": "boolean", "value": True}

    # Reload the page to cleanup the state
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=top_context["url"], wait="complete"
    )
