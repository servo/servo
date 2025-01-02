import pytest
from webdriver.bidi.modules.script import ContextTarget

from ... import recursive_compare
from .. import PRIMITIVE_VALUES


@pytest.mark.asyncio
async def test_default_arguments(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="(...args) => args",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare({"type": "array", "value": []}, result)


@pytest.mark.asyncio
@pytest.mark.parametrize("expected, argument", PRIMITIVE_VALUES)
async def test_primitive_value(bidi_session, top_context, argument, expected):
    result = await bidi_session.script.call_function(
        function_declaration=f"""(arg) => {{
            if (typeof {expected} === "number" && isNaN({expected})) {{
                if (!isNaN(arg)) {{
                    throw new Error(`Argument should be {expected}, but was ` + arg);
                }}
            }} else if (arg !== {expected}) {{
                throw new Error(`Argument should be {expected}, but was ` + arg);
            }}
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "argument, expected_type",
    [
        (
            {
                "type": "array",
                "value": [
                    {"type": "string", "value": "foobar"},
                ],
            },
            "Array",
        ),
        ({"type": "date", "value": "2022-05-31T13:47:29.000Z"}, "Date"),
        (
            {
                "type": "map",
                "value": [
                    ["foobar", {"type": "string", "value": "foobar"}],
                ],
            },
            "Map",
        ),
        (
            {
                "type": "object",
                "value": [
                    ["foobar", {"type": "string", "value": "foobar"}],
                ],
            },
            "Object",
        ),
        ({"type": "regexp", "value": {"pattern": "foo", "flags": "g"}}, "RegExp"),
        (
            {
                "type": "set",
                "value": [
                    {"type": "string", "value": "foobar"},
                ],
            },
            "Set",
        ),
    ],
)
async def test_local_value(bidi_session, top_context, argument, expected_type):
    result = await bidi_session.script.call_function(
        function_declaration=f"""(arg) => {{
            if (!(arg instanceof {expected_type})) {{
                const type = Object.prototype.toString.call(arg);
                throw new Error(
                    "Argument type should be {expected_type}, but was " + type
                );
            }}
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)
