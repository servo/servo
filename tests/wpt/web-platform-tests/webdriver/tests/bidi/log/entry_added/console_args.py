import pytest
from webdriver.bidi.modules.script import ContextTarget

from . import assert_console_entry, create_console_api_message_for_primitive_value


@pytest.mark.asyncio
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
    bidi_session, top_context, wait_for_event, data, remote_value
):
    await bidi_session.session.subscribe(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")
    await create_console_api_message_for_primitive_value(
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
