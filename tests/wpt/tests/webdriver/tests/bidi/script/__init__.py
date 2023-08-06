from __future__ import annotations
from typing import Any, Callable, Mapping
from .. import any_int, any_string, recursive_compare


def assert_handle(obj: Mapping[str, Any], should_contain_handle: bool) -> None:
    if should_contain_handle:
        assert "handle" in obj, f"Result should contain `handle`. Actual: {obj}"
        assert isinstance(obj["handle"], str), f"`handle` should be a string, but was {type(obj['handle'])}"

        # Recursively check that handle is not found in any of the nested values.
        if "value" in obj:
            value = obj["value"]
            if type(value) is list:
                for v in value:
                    assert_handle(v, False)

            if type(value) is dict:
                for v in value.values():
                    assert_handle(v, False)

    else:
        assert "handle" not in obj, f"Result should not contain `handle`. Actual: {obj}"


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
    ("window", {"type": "window", },),
    ("new URL('https://example.com')", {"type": "object", },),
]
