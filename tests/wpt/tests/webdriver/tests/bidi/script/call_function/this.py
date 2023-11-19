import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import any_string, recursive_compare


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
                    "value": 42,
                }]]},
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    assert result == {
        'type': 'number',
        'value': 42,
    }


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


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "channel, expected_data",
    [
        (
            {"type": "channel", "value": {"channel": "channel_name"}},
            {"type": "object", "value": [["foo", {"type": "string", "value": "bar"}]]},
        ),
        (
            {
                "type": "channel",
                "value": {
                    "channel": "channel_name",
                    "serializationOptions": {
                        "maxObjectDepth": 0,
                    },
                },
            },
            {"type": "object"},
        ),
        (
            {
                "type": "channel",
                "value": {"channel": "channel_name", "ownership": "root"},
            },
            {
                "handle": any_string,
                "type": "object",
                "value": [["foo", {"type": "string", "value": "bar"}]],
            },
        ),
    ],
    ids=["default", "with serializationOptions", "with ownership"],
)
async def test_channel(
    bidi_session, top_context, subscribe_events, wait_for_event,
    wait_for_future_safe, channel, expected_data
):
    await subscribe_events(["script.message"])

    on_entry_added = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="function() { return this({'foo': 'bar'}) }",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
        this=channel,
    )
    event_data = await wait_for_future_safe(on_entry_added)

    recursive_compare(
        {
            "channel": "channel_name",
            "data": expected_data,
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )
