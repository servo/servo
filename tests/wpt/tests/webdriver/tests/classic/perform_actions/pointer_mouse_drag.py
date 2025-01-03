import pytest

from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.classic.perform_actions.support.refine import get_events
from tests.support.sync import Poll


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
@pytest.mark.parametrize("dx, dy", [
    (20, 0), (0, 15), (10, 15), (-20, 0), (10, -15), (-10, -15)
])
def test_drag_and_drop(session,
                       test_actions_page,
                       mouse_chain,
                       dx,
                       dy,
                       drag_duration):
    drag_target = session.find.css("#dragTarget", all=False)
    initial_rect = drag_target.rect
    initial_center = get_inview_center(
        initial_rect, get_viewport_rect(session))

    # Conclude chain with extra move to allow time for last queued
    # coordinate-update of drag_target and to test that drag_target is "dropped".
    mouse_chain \
        .pointer_move(0, 0, origin=drag_target) \
        .pointer_down() \
        .pointer_move(dx, dy, duration=drag_duration, origin="pointer") \
        .pointer_up() \
        .pointer_move(80, 50, duration=100, origin="pointer") \
        .perform()

    # mouseup that ends the drag is at the expected destination
    e = get_events(session)[1]
    assert e["type"] == "mouseup"
    assert e["pageX"] == pytest.approx(initial_center["x"] + dx, abs=1.0)
    assert e["pageY"] == pytest.approx(initial_center["y"] + dy, abs=1.0)

    final_rect = None

    def check_final_position(_):
        nonlocal final_rect

        final_rect = drag_target.rect
        return (
            final_rect["x"] == pytest.approx(
                initial_rect["x"] + dx, abs=1.0) and
            final_rect["y"] == pytest.approx(
                initial_rect["y"] + dy, abs=1.0)

        )

    wait = Poll(
        session, message="""Dragged element did not reach target position""")
    wait.until(check_final_position)

    assert final_rect["x"] == pytest.approx(
        initial_rect["x"] + dx, abs=1.0)
    assert final_rect["y"] == pytest.approx(
        initial_rect["y"] + dy, abs=1.0)


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
def test_drag_and_drop_with_draggable_element(session_new_window,
                                              test_actions_page,
                                              mouse_chain,
                                              drag_duration):
    new_session = session_new_window
    drag_target = new_session.find.css("#draggable", all=False)
    drop_target = new_session.find.css("#droppable", all=False)
    # Conclude chain with extra move to allow time for last queued
    # coordinate-update of drag_target and to test that drag_target is "dropped".
    mouse_chain \
        .pointer_move(0, 0, origin=drag_target) \
        .pointer_down() \
        .pointer_move(50,
                      25,
                      duration=drag_duration,
                      origin=drop_target) \
        .pointer_up() \
        .pointer_move(80, 50, duration=100, origin="pointer") \
        .perform()
    # mouseup that ends the drag is at the expected destination
    e = get_events(new_session)
    assert len(e) >= 5
    assert e[1]["type"] == "dragstart", "Events captured were {}".format(e)
    assert e[2]["type"] == "dragover", "Events captured were {}".format(e)
    drag_events_captured = [
        ev["type"] for ev in e if ev["type"].startswith("drag") or ev["type"].startswith("drop")
    ]
    assert "dragend" in drag_events_captured
    assert "dragenter" in drag_events_captured
    assert "dragleave" in drag_events_captured
    assert "drop" in drag_events_captured
