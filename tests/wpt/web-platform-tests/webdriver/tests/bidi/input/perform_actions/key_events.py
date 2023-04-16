import pytest

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_supported_key_events
from .. import get_events

pytestmark = pytest.mark.asyncio


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
async def test_printable_key_sends_correct_events(
    bidi_session, top_context, test_actions_page_bidi, get_focused_key_input, value, code
):
    await test_actions_page_bidi()
    elem = await get_focused_key_input()

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

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    keys_value = await bidi_session.script.call_function(function_declaration="""
(elem) => {
  return elem.value
}""",
                                                        target=ContextTarget(top_context["context"]),
                                                        arguments=[elem],
                                                        await_promise=False)

    assert keys_value["value"] == value
