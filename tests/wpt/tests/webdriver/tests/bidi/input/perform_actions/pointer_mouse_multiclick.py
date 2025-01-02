import pytest

from webdriver.bidi.modules.input import Actions
from webdriver.bidi.modules.script import ContextTarget

from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict

from .. import get_events
from . import get_element_rect

pytestmark = pytest.mark.asyncio


_DBLCLICK_INTERVAL = 640


@pytest.mark.parametrize("pause_during_click", [True, False])
@pytest.mark.parametrize("click_pause", [0, 200, _DBLCLICK_INTERVAL + 10])
async def test_dblclick_at_coordinates(
    bidi_session, top_context, load_static_test_page, pause_during_click, click_pause
):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }
    actions = Actions()
    input_source = (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"])
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    # Either pause before the second click, which might prevent the double click
    # depending on the pause delay. Or between mousedown and mouseup for the
    # second click, which will never prevent a double click.
    if pause_during_click:
        input_source.pointer_down(button=0).pause(duration=click_pause)
    else:
        input_source.pause(duration=click_pause).pointer_down(button=0)

    input_source.pointer_up(button=0)

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    # mouseup that ends the drag is at the expected destination
    events = await get_events(bidi_session, top_context["context"])

    assert_move_to_coordinates(div_point, "outer", events)

    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
    ]

    if pause_during_click or click_pause < _DBLCLICK_INTERVAL:
        expected.append({"type": "dblclick", "button": 0})

    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


async def test_no_dblclick_when_mouse_moves(
    bidi_session, top_context, load_static_test_page
):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"])
        .pointer_down(button=0)
        .pointer_up(button=0)
        .pointer_move(x=div_point["x"] + 10, y=div_point["y"] + 10)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])

    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
    ]

    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


lots_of_text = (
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor "
    "incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud "
    "exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."
)


async def test_tripleclick_at_coordinates(
    bidi_session, top_context, inline, get_element
):
    """
    This test does a triple click on a coordinate. On desktop platforms
    this will select a paragraph. On mobile this will not have the same
    desired outcome as taps are handled differently on mobile.
    """
    url = inline(
        f"""<div>{lots_of_text}</div>"""
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    div = await get_element("div")
    div_rect = await get_element_rect(bidi_session, context=top_context, element=div)
    div_centre = {
        "x": div_rect["x"] + div_rect["width"] / 2,
        "y": div_rect["y"] + div_rect["height"] / 2,
    }

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=int(div_centre["x"]), y=int(div_centre["y"]))
        .pointer_down(button=0)
        .pointer_up(button=0)
        .pointer_down(button=0)
        .pointer_up(button=0)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    actual_text = await bidi_session.script.evaluate(
        expression="document.getSelection().toString()",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    assert actual_text["value"] == lots_of_text
