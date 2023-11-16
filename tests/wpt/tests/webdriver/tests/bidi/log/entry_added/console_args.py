import pytest

from . import assert_console_entry, create_console_api_message_from_string
from ... import any_string

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("data,remote_value", [
    ("undefined", {"type": "undefined"}),
    ("null", {"type": "null"}),
    ("'bar'", {"type": "string", "value": "bar"}),
    ("42", {"type": "number", "value": 42}),
    ("Number.NaN", {"type": "number", "value": "NaN"}),
    ("-0", {"type": "number", "value": "-0"}),
    ("Number.POSITIVE_INFINITY", {"type": "number", "value": "Infinity"}),
    ("Number.NEGATIVE_INFINITY", {"type": "number", "value": "-Infinity"}),
    ("false", {"type": "boolean", "value": False}),
    ("42n", {"type": "bigint", "value": "42"}),
], ids=[
    "undefined",
    "null",
    "string",
    "number",
    "NaN",
    "-0",
    "Infinity",
    "-Infinity",
    "boolean",
    "bigint",
])
async def test_primitive_types(
    bidi_session, subscribe_events, top_context, wait_for_event, data, remote_value
):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, "log", f"'foo', {data}")
    event_data = await on_entry_added
    args = [
        {"type": "string", "value": "foo"},
        {"type": remote_value["type"]},
    ]
    if "value" in remote_value:
        args[1].update({"value": remote_value["value"]})

    # First arg is always the first argument as provided to console.log()
    assert_console_entry(event_data, args=args)


@pytest.mark.parametrize(
    "data, remote_value",
    [
        (
            "(Symbol('foo'))",
            {
                "type": "symbol",
            },
        ),
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
                    {"type": "array", "value": [{"type": "number", "value": 1}]},
                ],
            },
        ),
        (
            "({'foo': {'bar': 'baz'}, 'qux': 'quux'})",
            {
                "type": "object",
                "value": [
                    ["foo", {"type": "object", "value": [['bar', {"type": "string", "value": "baz"}]]}],
                    ["qux", {"type": "string", "value": "quux"}],
                ],
            },
        ),
        (
            "(function(){})",
            {
                "type": "function",
            },
        ),
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
                    [
                        "baz",
                        {"type": "array", "value": [{"type": "number", "value": 1}]},
                    ],
                ],
            },
        ),
        (
            "new Set([1, 'foo', true, [1]])",
            {
                "type": "set",
                "value": [
                    {"type": "number", "value": 1},
                    {"type": "string", "value": "foo"},
                    {"type": "boolean", "value": True},
                    {"type": "array", "value": [{"type": "number", "value": 1}]},
                ],
            },
        ),
        (
            "new WeakMap()",
            {
                "type": "weakmap",
            },
        ),
        (
            "new WeakSet()",
            {
                "type": "weakset",
            },
        ),
        (
            "new Error('SOME_ERROR_TEXT')",
            {"type": "error"},
        ),
        (
            "Promise.resolve()",
            {
                "type": "promise",
            },
        ),
        (
            "new Int32Array()",
            {
                "type": "typedarray",
            },
        ),
        (
            "new ArrayBuffer()",
            {
                "type": "arraybuffer",
            },
        ),
        (
            "window",
            {
                "type": "window",
            },
        ),
        (
            "new URL('https://example.com')",
            {
                "type": "object",
            },
        ),
    ],
)
async def test_remote_values(
    bidi_session, subscribe_events, top_context, wait_for_event, data, remote_value
):
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, "log", data
    )
    event_data = await on_entry_added
    arg = {"type": remote_value["type"]}
    if "value" in remote_value:
        arg["value"] = remote_value["value"]

    # First arg is always the first argument as provided to console.log()
    assert_console_entry(event_data, args=[arg])


@pytest.mark.parametrize(
    "data, expected",
    [
        (
            "document.querySelector('br')",
            [
                {
                    "type": "node",
                    "sharedId": any_string,
                    "value": {
                        "nodeType": 1,
                        "localName": "br",
                        "namespaceURI": "http://www.w3.org/1999/xhtml",
                        "childNodeCount": 0,
                        "attributes": {},
                        "shadowRoot": None,
                    },
                },
            ],
        ),
        (
            "document.querySelector('#custom-element')",
            [
                {
                    "type": "node",
                    "sharedId": any_string,
                    "value": {
                        "attributes": {
                            "id": "custom-element",
                        },
                        "childNodeCount": 0,
                        "localName": "custom-element",
                        "namespaceURI": "http://www.w3.org/1999/xhtml",
                        "nodeType": 1,
                        "shadowRoot": {
                            "sharedId": any_string,
                            "type": "node",
                        },
                    },
                },
            ],
        ),
    ],
    ids=["basic", "shadowRoot"],
)
async def test_node(
    bidi_session, subscribe_events, get_test_page, top_context, wait_for_event, data, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=get_test_page(), wait="complete"
    )
    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_from_string(
        bidi_session, top_context, "log", data
    )
    event_data = await on_entry_added

    assert_console_entry(event_data, args=expected)
