import pytest

from webdriver.bidi.modules.input import Actions

from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict

from .. import get_events
from . import get_element_rect, get_inview_center_bidi

pytestmark = pytest.mark.asyncio


async def test_click_at_coordinates(bidi_session, top_context, load_static_test_page):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"], duration=1000)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])

    assert len(events) == 4
    assert_move_to_coordinates(div_point, "outer", events)

    for e in events:
        if e["type"] != "mousedown":
            assert e["buttons"] == 0
        assert e["button"] == 0

    expected = [
        {"type": "mousedown", "buttons": 1},
        {"type": "mouseup", "buttons": 0},
        {"type": "click", "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


async def test_context_menu_at_coordinates(
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
        .pointer_down(button=2)
        .pointer_up(button=2)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 4

    expected = [
        {"type": "mousedown", "button": 2},
        {"type": "contextmenu", "button": 2},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_contextmenu_events = [
        x for x in filtered_events if x["type"] in ["mousedown", "contextmenu"]
    ]
    assert expected == mousedown_contextmenu_events


async def test_click_element_center(
    bidi_session, top_context, get_element, load_static_test_page
):
    await load_static_test_page(page="test_actions.html")

    outer = await get_element("#outer")
    center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=outer
    )

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=outer)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 4

    event_types = [e["type"] for e in events]
    assert ["mousemove", "mousedown", "mouseup", "click"] == event_types
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == pytest.approx(center["x"], abs=1.0)
            assert e["pageY"] == pytest.approx(center["y"], abs=1.0)
            assert e["target"] == "outer"


async def test_click_navigation(
    bidi_session,
    top_context,
    url,
    inline,
    subscribe_events,
    wait_for_event,
    get_element,
):
    await subscribe_events(events=["browsingContext.load"])

    destination = url("/webdriver/tests/support/html/test_actions.html")
    start = inline(f'<a href="{destination}" id="link">destination</a>')

    async def click_link():
        link = await get_element("#link")

        actions = Actions()
        (
            actions.add_pointer()
            .pointer_move(x=0, y=0, origin=link)
            .pointer_down(button=0)
            .pointer_up(button=0)
        )
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )

    # repeat steps to check behaviour after document unload
    for _ in range(2):
        await bidi_session.browsing_context.navigate(
            context=top_context["context"], url=start, wait="complete"
        )

        on_entry = wait_for_event("browsingContext.load")
        await click_link()
        event = await on_entry
        assert event["url"] == destination


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
@pytest.mark.parametrize(
    "dx, dy", [(20, 0), (0, 15), (10, 15), (-20, 0), (10, -15), (-10, -15)]
)
async def test_drag_and_drop(
    bidi_session,
    top_context,
    get_element,
    load_static_test_page,
    dx,
    dy,
    drag_duration,
):
    await load_static_test_page(page="test_actions.html")

    drag_target = await get_element("#dragTarget")
    initial_rect = await get_element_rect(
        bidi_session, context=top_context, element=drag_target
    )
    initial_center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=drag_target
    )

    # Conclude chain with extra move to allow time for last queued
    # coordinate-update of drag_target and to test that drag_target is "dropped".
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=drag_target)
        .pointer_down(button=0)
        .pointer_move(dx, dy, duration=drag_duration, origin="pointer")
        .pointer_up(button=0)
        .pointer_move(80, 50, duration=100, origin="pointer")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    # mouseup that ends the drag is at the expected destination
    events = await get_events(bidi_session, top_context["context"])
    e = events[1]
    assert e["type"] == "mouseup"
    assert e["pageX"] == pytest.approx(initial_center["x"] + dx, abs=1.0)
    assert e["pageY"] == pytest.approx(initial_center["y"] + dy, abs=1.0)
    # check resulting location of the dragged element
    final_rect = await get_element_rect(
        bidi_session, context=top_context, element=drag_target
    )
    assert initial_rect["x"] + dx == final_rect["x"]
    assert initial_rect["y"] + dy == final_rect["y"]


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
async def test_drag_and_drop_with_draggable_element(
    bidi_session, top_context, get_element, load_static_test_page, drag_duration
):
    new_context = await bidi_session.browsing_context.create(type_hint="window")
    await load_static_test_page(page="test_actions.html", context=new_context)
    drag_target = await get_element("#draggable")
    drop_target = await get_element("#droppable")

    # Conclude chain with extra move to allow time for last queued
    # coordinate-update of drag_target and to test that drag_target is "dropped".
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=drag_target)
        .pointer_down(button=0)
        .pointer_move(x=50, y=25, duration=drag_duration, origin=drop_target)
        .pointer_up(button=0)
        .pointer_move(80, 50, duration=100, origin="pointer")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=new_context["context"]
    )

    # mouseup that ends the drag is at the expected destination
    events = await get_events(bidi_session, new_context["context"])

    assert len(events) >= 5
    assert events[1]["type"] == "dragstart", f"Events captured were {events}"
    assert events[2]["type"] == "dragover", f"Events captured were {events}"
    drag_events_captured = [
        ev["type"]
        for ev in events
        if ev["type"].startswith("drag") or ev["type"].startswith("drop")
    ]
    assert "dragend" in drag_events_captured
    assert "dragenter" in drag_events_captured
    assert "dragleave" in drag_events_captured
    assert "drop" in drag_events_captured
