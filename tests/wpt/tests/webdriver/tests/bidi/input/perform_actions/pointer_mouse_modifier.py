import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from tests.support.helpers import filter_dict
from tests.support.keys import Keys

from .. import get_events

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "modifier, prop",
    [
        (Keys.CONTROL, "ctrlKey"),
        (Keys.R_CONTROL, "ctrlKey"),
    ],
)
async def test_control_click(
    bidi_session,
    current_session,
    top_context,
    get_element,
    load_static_test_page,
    modifier,
    prop,
):
    os = current_session.capabilities["platformName"]

    await load_static_test_page(page="test_actions.html")
    outer = await get_element("#outer")

    actions = Actions()
    (
        actions.add_key()
        .pause(duration=0)
        .key_down(modifier)
        .pause(duration=200)
        .key_up(modifier)
    )
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(outer))
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    if os != "mac":
        expected = [
            {"type": "mousemove"},
            {"type": "mousedown"},
            {"type": "mouseup"},
            {"type": "click"},
        ]
    else:
        expected = [
            {"type": "mousemove"},
            {"type": "mousedown"},
            {"type": "contextmenu"},
            {"type": "mouseup"},
        ]

    defaults = {"altKey": False, "metaKey": False, "shiftKey": False, "ctrlKey": False}

    for e in expected:
        e.update(defaults)
        if e["type"] != "mousemove":
            e[prop] = True

    all_events = await get_events(bidi_session, top_context["context"])
    filtered_events = [filter_dict(e, expected[0]) for e in all_events]
    assert expected == filtered_events


async def test_control_click_release(
    bidi_session, top_context, load_static_test_page, get_focused_key_input
):
    await load_static_test_page(page="test_actions.html")
    key_reporter = await get_focused_key_input()

    # The context menu stays visible during subsequent tests so let's not
    # display it in the first place.
    await bidi_session.script.evaluate(
        expression="""
            var keyReporter = document.getElementById("keys");
            document.addEventListener("contextmenu", function(e) {
              e.preventDefault();
            });
        """,
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    actions = Actions()
    actions.add_key().pause(duration=0).key_down(Keys.CONTROL)
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(key_reporter))
        .pointer_down(button=0)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    await bidi_session.script.evaluate(
        expression="""
            var keyReporter = document.getElementById("keys");
            keyReporter.addEventListener("mousedown", recordPointerEvent);
            keyReporter.addEventListener("mouseup", recordPointerEvent);
            resetEvents();
        """,
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    await bidi_session.input.release_actions(context=top_context["context"])

    expected = [
        {"type": "mouseup"},
        {"type": "keyup"},
    ]
    all_events = await get_events(bidi_session, top_context["context"])
    events = [filter_dict(e, expected[0]) for e in all_events]
    assert events == expected


async def test_many_modifiers_click(
    bidi_session, top_context, get_element, load_static_test_page
):
    await load_static_test_page(page="test_actions.html")
    outer = await get_element("#outer")

    dblclick_timeout = 800
    actions = Actions()
    (
        actions.add_key()
        .pause(duration=0)
        .key_down(Keys.ALT)
        .key_down(Keys.SHIFT)
        .pause(duration=dblclick_timeout)
        .key_up(Keys.ALT)
        .key_up(Keys.SHIFT)
    )
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(outer))
        .pause(duration=0)
        .pointer_down(button=0)
        .pointer_up(button=0)
        .pause(duration=0)
        .pause(duration=0)
        .pointer_down(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    expected = [
        {"type": "mousemove"},
        # shift and alt pressed
        {"type": "mousedown"},
        {"type": "mouseup"},
        {"type": "click"},
        # no modifiers pressed
        {"type": "mousedown"},
    ]

    defaults = {"altKey": False, "metaKey": False, "shiftKey": False, "ctrlKey": False}

    for e in expected:
        e.update(defaults)

    for e in expected[1:4]:
        e["shiftKey"] = True
        e["altKey"] = True

    all_events = await get_events(bidi_session, top_context["context"])
    events = [filter_dict(e, expected[0]) for e in all_events]
    assert events == expected


@pytest.mark.parametrize(
    "modifier, prop",
    [
        (Keys.ALT, "altKey"),
        (Keys.R_ALT, "altKey"),
        (Keys.META, "metaKey"),
        (Keys.R_META, "metaKey"),
        (Keys.SHIFT, "shiftKey"),
        (Keys.R_SHIFT, "shiftKey"),
    ],
)
async def test_modifier_click(
    bidi_session, top_context, get_element, load_static_test_page, modifier, prop
):
    await load_static_test_page(page="test_actions.html")
    outer = await get_element("#outer")

    actions = Actions()
    (
        actions.add_key()
        .pause(duration=200)
        .key_down(modifier)
        .pause(duration=200)
        .pause(duration=0)
        .key_up(modifier)
    )
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(outer))
        .pause(duration=50)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    expected = [
        {"type": "mousemove"},
        {"type": "mousedown"},
        {"type": "mouseup"},
        {"type": "click"},
    ]

    defaults = {"altKey": False, "metaKey": False, "shiftKey": False, "ctrlKey": False}

    for e in expected:
        e.update(defaults)
        if e["type"] != "mousemove":
            e[prop] = True

    all_events = await get_events(bidi_session, top_context["context"])
    filtered_events = [filter_dict(e, expected[0]) for e in all_events]
    assert expected == filtered_events
