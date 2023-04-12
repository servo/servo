import json

import pytest

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_dict

pytestmark = pytest.mark.asyncio


async def get_events(context, bidi_session):
    """Return list of key events recorded in the test_keys_page fixture."""
    events_str = await bidi_session.script.evaluate(expression="JSON.stringify(allEvents.events)",
                                                    target=ContextTarget(context),
                                                    await_promise=False)
    events = json.loads(events_str["value"])

    # `key` values in `allEvents` may be escaped (see `escapeSurrogateHalf` in
    # test_actions.html), so this converts them back into unicode literals.
    for e in events:
        # example: turn "U+d83d" (6 chars) into u"\ud83d" (1 char)
        if "key" in e and e["key"].startswith(u"U+"):
            key = e["key"]
            hex_suffix = key[key.index("+") + 1:]
            e["key"] = chr(int(hex_suffix, 16))

        # WebKit sets code as 'Unidentified' for unidentified key codes, but
        # tests expect ''.
        if "code" in e and e["code"] == "Unidentified":
            e["code"] = ""
    return events


@pytest.mark.parametrize("value,code", [
    (u"a", "KeyA",),
    ("a", "KeyA",),
    (u"\"", "Quote"),
    (u",", "Comma"),
    (u"\u00E0", ""),
    (u"\u0416", ""),
    (u"@", "Digit2"),
    (u"\u2603", ""),
    (u"\uF6C2", ""),  # PUA
])
async def test_printable_key_sends_correct_events(bidi_session, top_context, url, value, code):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url("/webdriver/tests/support/html/test_actions.html"),
        wait="complete"
    )
    await bidi_session.script.call_function(function_declaration="""
() => {
  let elem = document.getElementById("keys");
  elem.focus();
  resetEvents();
}""",
                                           target=ContextTarget(top_context["context"]),
                                           await_promise=False)

    actions = Actions()
    (actions.add_key()
     .key_down(value)
     .key_up(value))
    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    all_events = await get_events(top_context["context"], bidi_session)

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keypress"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    events = [filter_dict(e, expected[0]) for e in all_events]
    if len(events) > 0 and events[0]["code"] is None:
        # Remove 'code' entry if browser doesn't support it
        expected = [filter_dict(e, {"key": "", "type": ""}) for e in expected]
        events = [filter_dict(e, expected[0]) for e in events]
    assert events == expected

    keys_value = await bidi_session.script.call_function(function_declaration="""
() => {
  let elem = document.getElementById("keys");
  return elem.value
}""",
                                                        target=ContextTarget(top_context["context"]),
                                                        await_promise=False)

    assert keys_value["value"] == value
