# META: timeout=long

import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin

from .. import get_events
from . import get_element_rect, get_inview_center_bidi

pytestmark = pytest.mark.asyncio


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
        .pointer_move(x=0, y=0, origin=get_element_origin(drag_target))
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
    assert final_rect["x"] == pytest.approx(initial_rect["x"] + dx, abs=1.0)
    assert final_rect["y"] == pytest.approx(initial_rect["y"] + dy, abs=1.0)


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
async def test_drag_and_drop_with_draggable_element(bidi_session, top_context,
                                                    get_element,
                                                    load_static_test_page,
                                                    drag_duration):
    await load_static_test_page(page="test_actions.html")

    drag_target = await get_element("#draggable")
    drop_target = await get_element("#droppable")

    # Conclude chain with extra move to allow time for last queued
    # coordinate-update of drag_target and to test that drag_target is "dropped".
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(drag_target))
        .pointer_down(button=0)
        .pointer_move(x=0, y=0, duration=drag_duration, origin=get_element_origin(drop_target))
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(actions=actions,
                                             context=top_context["context"])

    # mouseup that ends the drag is at the expected destination
    events = await get_events(bidi_session, top_context["context"])

    drag_events_captured = [
        ev["type"] for ev in events
        if ev["type"].startswith("drag") or ev["type"].startswith("drop")
    ]
    assert "dragstart" in drag_events_captured
    assert "dragenter" in drag_events_captured
    # dragleave never happens if the mouse moves directly into the drop element
    # without intermediate movements.
    if drag_duration != 0:
        assert "dragleave" in drag_events_captured
    assert "dragover" in drag_events_captured
    assert "drop" in drag_events_captured
    assert "dragend" in drag_events_captured

    def last_index(list, value):
        return len(list) - list[::-1].index(value) - 1

    # The order should follow the diagram:
    #
    #  - dragstart
    #  - dragenter
    #  - ...
    #  - dragenter
    #  - dragleave
    #  - ...
    #  - dragleave
    #  - dragover
    #  - ...
    #  - dragover
    #  - drop
    #  - dragend
    #
    assert drag_events_captured.index(
        "dragstart") < drag_events_captured.index("dragenter")
    if drag_duration != 0:
        assert last_index(drag_events_captured,
                      "dragenter") < last_index(drag_events_captured, "dragleave")
        assert last_index(drag_events_captured,
                      "dragleave") < last_index(drag_events_captured, "dragover")
    else:
        assert last_index(drag_events_captured,
                      "dragenter") < last_index(drag_events_captured, "dragover")
    assert last_index(drag_events_captured,
                  "dragover") < drag_events_captured.index("drop")
    assert drag_events_captured.index(
        "drop") == drag_events_captured.index("dragend") - 1
