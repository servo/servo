import json

from webdriver.bidi.modules.script import ContextTarget


async def get_events(bidi_session, context):
    """Return list of key events recorded on the test_actions.html page."""
    events_str = await bidi_session.script.evaluate(
        expression="JSON.stringify(allEvents.events)",
        target=ContextTarget(context),
        await_promise=False,
    )
    events = json.loads(events_str["value"])

    # `key` values in `allEvents` may be escaped (see `escapeSurrogateHalf` in
    # test_actions.html), so this converts them back into unicode literals.
    for e in events:
        # example: turn "U+d83d" (6 chars) into u"\ud83d" (1 char)
        if "key" in e and e["key"].startswith("U+"):
            key = e["key"]
            hex_suffix = key[key.index("+") + 1:]
            e["key"] = chr(int(hex_suffix, 16))

        # WebKit sets code as 'Unidentified' for unidentified key codes, but
        # tests expect ''.
        if "code" in e and e["code"] == "Unidentified":
            e["code"] = ""
    return events


async def get_keys_value(bidi_session, context):
    keys_value = await bidi_session.script.evaluate(
        expression="""document.getElementById("keys").value""",
        target=ContextTarget(context),
        await_promise=False,
    )

    return keys_value["value"]
