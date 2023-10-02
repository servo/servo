# META: timeout=long
import copy
import pytest

from collections import defaultdict

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_dict, filter_supported_key_events
from tests.support.keys import ALL_EVENTS, Keys, ALTERNATIVE_KEY_NAMES
from .. import get_events, get_keys_value

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "key,event",
    [
        (Keys.ESCAPE, "ESCAPE"),
        (Keys.RIGHT, "RIGHT"),
    ],

)
async def test_non_printable_key_sends_events(
    bidi_session, top_context, key, event
):
    code = ALL_EVENTS[event]["code"]
    value = ALL_EVENTS[event]["key"]

    actions = Actions()
    (actions.add_key().key_down(key).key_up(key))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    all_events = await get_events(bidi_session, top_context["context"])

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keypress"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    # Make a copy for alternate key property values
    # Note: only keydown and keyup are affected by alternate key names
    alt_expected = copy.deepcopy(expected)
    if event in ALTERNATIVE_KEY_NAMES:
        alt_expected[0]["key"] = ALTERNATIVE_KEY_NAMES[event]
        alt_expected[2]["key"] = ALTERNATIVE_KEY_NAMES[event]

    (_, expected) = filter_supported_key_events(all_events, expected)
    (events, alt_expected) = filter_supported_key_events(all_events, alt_expected)
    if len(events) == 2:
        # most browsers don't send a keypress for non-printable keys
        assert events == [expected[0], expected[2]] or events == [
            alt_expected[0],
            alt_expected[2],
        ]
    else:
        assert events == expected or events == alt_expected

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert len(keys_value) == 0


@pytest.mark.parametrize(
    "key, event",
    [
        (Keys.ALT, "ALT"),
        (Keys.CONTROL, "CONTROL"),
        (Keys.META, "META"),
        (Keys.SHIFT, "SHIFT"),
        (Keys.R_ALT, "R_ALT"),
        (Keys.R_CONTROL, "R_CONTROL"),
        (Keys.R_META, "R_META"),
        (Keys.R_SHIFT, "R_SHIFT"),
    ],
)
async def test_key_modifier_key(bidi_session, top_context, setup_key_test, key, event):
    code = ALL_EVENTS[event]["code"]
    value = ALL_EVENTS[event]["key"]

    actions = Actions()
    (actions.add_key().key_down(key).key_up(key))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    all_events = await get_events(bidi_session, top_context["context"])

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert len(keys_value) == 0


@pytest.mark.parametrize(
    "value,code",
    [
        ("a", "KeyA"),
        ("a", "KeyA"),
        ('"', "Quote"),
        (",", "Comma"),
        ("\u00E0", ""),
        ("\u0416", ""),
        ("@", "Digit2"),
        ("\u2603", ""),
        ("\uF6C2", ""),  # PUA
    ],
)
async def test_key_printable_key(
    bidi_session,
    top_context,
    setup_key_test,
    value,
    code,
):
    actions = Actions()
    (actions.add_key().key_down(value).key_up(value))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    all_events = await get_events(bidi_session, top_context["context"])

    expected = [
        {"code": code, "key": value, "type": "keydown"},
        {"code": code, "key": value, "type": "keypress"},
        {"code": code, "key": value, "type": "keyup"},
    ]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == value


@pytest.mark.parametrize("use_keyup", [True, False])
async def test_key_printable_sequence(bidi_session, top_context, use_keyup):
    actions = Actions()
    actions.add_key()
    if use_keyup:
        actions.add_key().send_keys("ab")
    else:
        actions.add_key().key_down("a").key_down("b")

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    all_events = await get_events(bidi_session, top_context["context"])

    expected = [
        {"code": "KeyA", "key": "a", "type": "keydown"},
        {"code": "KeyA", "key": "a", "type": "keypress"},
        {"code": "KeyA", "key": "a", "type": "keyup"},
        {"code": "KeyB", "key": "b", "type": "keydown"},
        {"code": "KeyB", "key": "b", "type": "keypress"},
        {"code": "KeyB", "key": "b", "type": "keyup"},
    ]
    expected = [e for e in expected if use_keyup or e["type"] != "keyup"]

    (events, expected) = filter_supported_key_events(all_events, expected)
    assert events == expected

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert keys_value == "ab"


@pytest.mark.parametrize("name,expected", ALL_EVENTS.items())
async def test_key_special_key_sends_keydown(
    bidi_session,
    top_context,
    setup_key_test,
    name,
    expected,
):
    if name.startswith("F"):
        # Prevent default behavior for F1, etc., but only after keydown
        # bubbles up to body. (Otherwise activated browser menus/functions
        # may interfere with subsequent tests.)
        await bidi_session.script.evaluate(
            expression="""
            document.body.addEventListener("keydown",
                function(e) { e.preventDefault() });
            """,
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )

    actions = Actions()
    (actions.add_key().key_down(getattr(Keys, name)))
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    # only interested in keydown
    all_events = await get_events(bidi_session, top_context["context"])
    first_event = all_events[0]
    # make a copy so we can throw out irrelevant keys and compare to events
    expected = dict(expected)

    del expected["value"]

    # make another copy for alternative key names
    alt_expected = copy.deepcopy(expected)
    if name in ALTERNATIVE_KEY_NAMES:
        alt_expected["key"] = ALTERNATIVE_KEY_NAMES[name]

    # check and remove keys that aren't in expected
    assert first_event["type"] == "keydown"
    assert first_event["repeat"] is False
    first_event = filter_dict(first_event, expected)
    if first_event["code"] is None:
        del first_event["code"]
        del expected["code"]
        del alt_expected["code"]
    assert first_event == expected or first_event == alt_expected
    # only printable characters should be recorded in input field
    keys_value = await get_keys_value(bidi_session, top_context["context"])
    if len(expected["key"]) == 1:
        assert keys_value == expected["key"]
    else:
        assert len(keys_value) == 0


async def test_key_space(bidi_session, top_context):
    actions = Actions()
    (
        actions.add_key()
        .key_down(Keys.SPACE)
        .key_up(Keys.SPACE)
        .key_down(" ")
        .key_up(" ")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )
    all_events = await get_events(bidi_session, top_context["context"])

    by_type = defaultdict(list)
    for event in all_events:
        by_type[event["type"]].append(event)

    for event_type in by_type:
        events = by_type[event_type]
        assert len(events) == 2
        assert events[0] == events[1]


async def test_keyup_only_sends_no_events(bidi_session, top_context):
    actions = Actions()
    actions.add_key().key_up("a")
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 0

    keys_value = await get_keys_value(bidi_session, top_context["context"])
    assert len(keys_value) == 0
