import json

from webdriver.bidi.modules.script import ContextTarget


async def add_mouse_listeners(bidi_session, context, include_mousemove=True):
    result = await bidi_session.script.call_function(
        function_declaration="""(include_mousemove) => {
            window.allEvents = { events: []};

            const events = ["auxclick", "click", "mousedown", "mouseup"];
            if (include_mousemove) {
              events.push("mousemove");
            }

            function handleEvent(event) {
                window.allEvents.events.push({
                    type: event.type,
                    detail: event.detail,
                    clientX: event.clientX,
                    clientY: event.clientY,
                    isTrusted: event.isTrusted,
                    button: event.button,
                    buttons: event.buttons,
                });
            };

            for (const event of events) {
                document.addEventListener(event, handleEvent);
            }
            }""",
        arguments=[{"type": "boolean", "value": include_mousemove}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )


async def get_object_from_context(bidi_session, context, object_path):
    """Return a plain JS object from a given context, accessible at the given object_path"""
    events_str = await bidi_session.script.evaluate(
        expression=f"JSON.stringify({object_path})",
        target=ContextTarget(context),
        await_promise=False,
    )
    return json.loads(events_str["value"])


async def get_events(bidi_session, context):
    """Return list of key events recorded on the test_actions.html page."""
    events = await get_object_from_context(bidi_session, context, "allEvents.events")

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
