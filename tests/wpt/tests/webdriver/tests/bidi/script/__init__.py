from __future__ import annotations
from typing import Any, Callable, Mapping
from webdriver.bidi.modules.script import ContextTarget

from .. import any_int, any_string, recursive_compare


def specific_error_response(expected_error: Mapping[str, Any]) -> Callable[[Any], None]:
    return lambda actual: recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": expected_error,
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        actual)


def any_stack_trace(actual: Any) -> None:
    assert type(actual) is dict
    assert "callFrames" in actual
    assert type(actual["callFrames"]) is list
    for actual_frame in actual["callFrames"]:
        any_stack_frame(actual_frame)


def any_stack_frame(actual: Any) -> None:
    assert type(actual) is dict

    assert "columnNumber" in actual
    any_int(actual["columnNumber"])

    assert "functionName" in actual
    any_string(actual["functionName"])

    assert "lineNumber" in actual
    any_int(actual["lineNumber"])

    assert "url" in actual
    any_string(actual["url"])


"""Format: List[(expression, expected)]"""
PRIMITIVE_VALUES: list[tuple[str, dict]] = [
    ("undefined", {"type": "undefined"}),
    ("null", {"type": "null"}),
    ("'foobar'", {"type": "string", "value": "foobar"}),
    ("'2'", {"type": "string", "value": "2"}),
    ("NaN", {"type": "number", "value": "NaN"}),
    ("-0", {"type": "number", "value": "-0"}),
    ("Infinity", {"type": "number", "value": "Infinity"}),
    ("-Infinity", {"type": "number", "value": "-Infinity"}),
    ("3", {"type": "number", "value": 3}),
    ("1.4", {"type": "number", "value": 1.4}),
    ("true", {"type": "boolean", "value": True}),
    ("false", {"type": "boolean", "value": False}),
    ("42n", {"type": "bigint", "value": "42"}),
]


"""Format: List[(expression, expected)]"""
REMOTE_VALUES: list[tuple[str, dict]] = [
    ("(Symbol('foo'))", {"type": "symbol", },),
    (
        "[1, 'foo', true, new RegExp(/foo/g), [1]]",
        {
            "type": "array",
            "value": [
                {"type": "number", "value": 1},
                {"type": "string", "value": "foo"},
                {"type": "boolean", "value": True},
                {
                    "type": "regexp",
                    "value": {
                        "pattern": "foo",
                        "flags": "g",
                    },
                },
                {"type": "array"},
            ],
        },
    ),
    (
        "({'foo': {'bar': 'baz'}, 'qux': 'quux', 1: 'fred', '2': 'thud'})",
        {
            "type": "object",
            "value": [
                ["1", {"type": "string", "value": "fred"}],
                ["2", {"type": "string", "value": "thud"}],
                ["foo", {"type": "object"}],
                ["qux", {"type": "string", "value": "quux"}],
            ],
        },
    ),
    ("(()=>{})", {"type": "function", },),
    ("(function(){})", {"type": "function", },),
    ("(async ()=>{})", {"type": "function", },),
    ("(async function(){})", {"type": "function", },),
    ("(function*() { yield 'a'; })", {
        "type": "function",
    }),
    (
        "new RegExp(/foo/g)",
        {
            "type": "regexp",
            "value": {
                "pattern": "foo",
                "flags": "g",
            },
        },
    ),
    (
        "new Date(1654004849000)",
        {
            "type": "date",
            "value": "2022-05-31T13:47:29.000Z",
        },
    ),
    (
        "new Map([[1, 2], ['foo', 'bar'], [true, false], ['baz', [1]]])",
        {
            "type": "map",
            "value": [
                [
                    {"type": "number", "value": 1},
                    {"type": "number", "value": 2},
                ],
                ["foo", {"type": "string", "value": "bar"}],
                [
                    {"type": "boolean", "value": True},
                    {"type": "boolean", "value": False},
                ],
                ["baz", {"type": "array"}],
            ],
        },
    ),
    (
        "new Set([1, 'foo', true, [1], new Map([[1,2]])])",
        {
            "type": "set",
            "value": [
                {"type": "number", "value": 1},
                {"type": "string", "value": "foo"},
                {"type": "boolean", "value": True},
                {"type": "array"},
                {"type": "map"},
            ],
        },
    ),
    ("new WeakMap()", {"type": "weakmap", },),
    ("new WeakSet()", {"type": "weakset", },),
    ("new Error('SOME_ERROR_TEXT')", {"type": "error"},),
    ("[1, 2][Symbol.iterator]()", {
        "type": "iterator",
    }),
    ("'mystring'[Symbol.iterator]()", {
        "type": "iterator",
    }),
    ("(new Set([1,2]))[Symbol.iterator]()", {
        "type": "iterator",
    }),
    ("(new Map([[1,2]]))[Symbol.iterator]()", {
        "type": "iterator",
    }),
    ("new Proxy({}, {})", {
        "type": "proxy",
    }),
    ("(function*() { yield 'a'; })()", {
        "type": "generator",
    }),
    ("(async function*() { yield await Promise.resolve(1); })()", {
        "type": "generator",
    }),
    ("Promise.resolve()", {"type": "promise", },),
    ("new Int32Array()", {"type": "typedarray", },),
    ("new ArrayBuffer()", {"type": "arraybuffer", },),
    (
        "document.createElement('div')",
        {
            "sharedId": any_string,
            "type": "node",
            'value': {
                'attributes': {},
                'childNodeCount': 0,
                'localName': 'div',
                'namespaceURI': 'http://www.w3.org/1999/xhtml',
                'nodeType': 1,
                'shadowRoot': None,
            }
        },
    ),
    (
        "window", {
            "type": "window",
            "value": {
                "context": any_string,
            }
        },
    ),
    ("new URL('https://example.com')", {"type": "object", },),
]


async def create_sandbox(bidi_session, context, sandbox_name="Test", method="evaluate"):
    if method == "evaluate":
        result = await bidi_session.script.evaluate(
            raw_result=True,
            expression="1 + 2",
            await_promise=False,
            target=ContextTarget(context, sandbox=sandbox_name),
        )
    elif method == "call_function":
        result = await bidi_session.script.call_function(
            raw_result=True,
            function_declaration="() => 1 + 2",
            await_promise=False,
            target=ContextTarget(context, sandbox=sandbox_name),
        )
    else:
        raise Exception(f"Unsupported method to create a sandbox: {method}")

    return result["realm"]
