import pytest

from webdriver.bidi.modules.script import ContextTarget
from ... import recursive_compare


@pytest.mark.asyncio
async def test_arguments(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="(...args)=>{return args}",
        arguments=[{
            "type": "string",
            "value": "ARGUMENT_STRING_VALUE"
        }, {
            "type": "number",
            "value": 42}],
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    recursive_compare({
        "type": "array",
        "value": [{
            "type": 'string',
            "value": 'ARGUMENT_STRING_VALUE'
        }, {
            "type": 'number',
            "value": 42}]},
        result)


@pytest.mark.asyncio
async def test_default_arguments(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="(...args)=>{return args}",
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    recursive_compare({
        "type": "array",
        "value": []
    }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "setup_expression, function_declaration, expected",
    [
        (
            "Symbol('foo')",
            "(symbol) => symbol.toString()",
            {"type": "string", "value": "Symbol(foo)"},
        ),
        ("[1,2]", "(array) => array[0]", {"type": "number", "value": 1}),
        (
            "new RegExp('foo')",
            "(regexp) => regexp.source",
            {"type": "string", "value": "foo"},
        ),
        (
            "new Date(1654004849000)",
            "(date) => date.toISOString()",
            {"type": "string", "value": "2022-05-31T13:47:29.000Z"},
        ),
        (
            "new Map([['foo', 'bar']])",
            "(map) => map.get('foo')",
            {"type": "string", "value": "bar"},
        ),
        (
            "new Set(['foo'])",
            "(set) => set.has('foo')",
            {"type": "boolean", "value": True},
        ),
        (
            "{const weakMap = new WeakMap(); weakMap.set(weakMap, 'foo')}",
            "(weakMap)=> weakMap.get(weakMap)",
            {"type": "string", "value": "foo"},
        ),
        (
            "{const weakSet = new WeakSet(); weakSet.add(weakSet)}",
            "(weakSet)=> weakSet.has(weakSet)",
            {"type": "boolean", "value": True},
        ),
        (
            "new Error('error message')",
            "(error) => error.message",
            {"type": "string", "value": "error message"},
        ),
        (
            "new SyntaxError('syntax error message')",
            "(error) => error.message",
            {"type": "string", "value": "syntax error message"},
        ),
        (
            "new Promise((resolve) => resolve(3))",
            "(promise) => promise",
            {"type": "number", "value": 3},
        ),
        (
            "new Int8Array(2)",
            "(int8Array) => int8Array.length",
            {"type": "number", "value": 2},
        ),
        (
            "new ArrayBuffer(8)",
            "(arrayBuffer) => arrayBuffer.byteLength",
            {"type": "number", "value": 8},
        ),
        ("() => true", "(func) => func()", {"type": "boolean", "value": True}),
        (
            "(function() {return false;})",
            "(func) => func()",
            {"type": "boolean", "value": False},
        ),
        (
             "window.foo = 3; window",
            "(window) => window.foo",
            {"type": "number", "value": 3},
        ),
        (
            "({SOME_PROPERTY:'SOME_VALUE'})",
            "(obj) => obj.SOME_PROPERTY",
            {"type": "string", "value": "SOME_VALUE"},
        ),
    ],
)
async def test_remote_value_argument(
    bidi_session, top_context, setup_expression, function_declaration, expected
):
    remote_value_result = await bidi_session.script.evaluate(
        expression=setup_expression,
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )
    remote_value_handle = remote_value_result.get("handle")

    assert isinstance(remote_value_handle, str)

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        arguments=[{"handle": remote_value_handle}],
        await_promise=True if remote_value_result["type"] == "promise" else False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "argument, expected",
    [
        ({"type": "undefined"}, "undefined"),
        ({"type": "null"}, "null"),
        ({"type": "string", "value": "foobar"}, "'foobar'"),
        ({"type": "string", "value": "2"}, "'2'"),
        ({"type": "number", "value": "-0"}, "-0"),
        ({"type": "number", "value": "Infinity"}, "Infinity"),
        ({"type": "number", "value": "-Infinity"}, "-Infinity"),
        ({"type": "number", "value": 3}, "3"),
        ({"type": "number", "value": 1.4}, "1.4"),
        ({"type": "boolean", "value": True}, "true"),
        ({"type": "boolean", "value": False}, "false"),
        ({"type": "bigint", "value": "42"}, "42n"),
    ],
)
async def test_primitive_values(bidi_session, top_context, argument, expected):
    result = await bidi_session.script.call_function(
        function_declaration=
        f"""(arg) => {{
            if(arg!=={expected})
                throw Error("Argument should be {expected}, but was "+arg);
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)


@pytest.mark.asyncio
async def test_nan(bidi_session, top_context):
    nan_remote_value = {"type": "number", "value": "NaN"}
    result = await bidi_session.script.call_function(
        function_declaration=
        f"""(arg) => {{
            if(!isNaN(arg))
                throw Error("Argument should be 'NaN', but was "+arg);
            return arg;
        }}""",
        arguments=[nan_remote_value],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(nan_remote_value, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "argument, expected_type",
    [
        ({
             "type": "array",
             "value": [
                 {"type": "string", "value": "foobar"},
             ],
         },
         "Array"
        ),
        ({"type": "date", "value": "2022-05-31T13:47:29.000Z"},
         "Date"
         ),
        ({
             "type": "map",
             "value": [
                 ["foobar", {"type": "string", "value": "foobar"}],
             ],
         },
         "Map"
        ),
        ({
             "type": "object",
             "value": [
                 ["foobar", {"type": "string", "value": "foobar"}],
             ],
         },
         "Object"
        ),
        ({"type": "regexp", "value": {"pattern": "foo", "flags": "g"}},
         "RegExp"
         ),
        ({
             "type": "set",
             "value": [
                 {"type": "string", "value": "foobar"},
             ],
         },
         "Set"
        )
    ],
)
async def test_local_values(bidi_session, top_context, argument, expected_type):
    result = await bidi_session.script.call_function(
        function_declaration=
        f"""(arg) => {{
            if(! (arg instanceof {expected_type}))
                throw Error("Argument type should be {expected_type}, but was "+
                    Object.prototype.toString.call(arg));
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "value_fn, function_declaration",
    [
        (
            lambda value: value,
            "function(arg) { return arg === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "object", "value": [["nested", value]]}),
            "function(arg) { return arg.nested === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "array", "value": [value]}),
            "function(arg) { return arg[0] === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "map", "value": [["foobar", value]]}),
            "function(arg) { return arg.get('foobar') === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "set", "value": [value]}),
            "function(arg) { return arg.has(window.SOME_OBJECT); }",
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

    # Check that a remote value can be successfully deserialized as an "argument"
    # parameter and compared against the original object in the page.
    result = await call_function(
        function_declaration=function_declaration,
        arguments=[value_fn(remote_value)],
    )
    assert result == {"type": "boolean", "value": True}

    # Reload the page to cleanup the state
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=top_context["url"], wait="complete"
    )
