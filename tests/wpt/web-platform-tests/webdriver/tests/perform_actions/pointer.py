# META: timeout=long

import pytest

from webdriver.error import NoSuchWindowException

from tests.perform_actions.support.mouse import get_inview_center, get_viewport_rect
from tests.perform_actions.support.refine import get_events
from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict
from tests.support.sync import Poll


def test_null_response_value(session, mouse_chain):
    value = mouse_chain.click().perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, mouse_chain):
    with pytest.raises(NoSuchWindowException):
        mouse_chain.click().perform()


def test_no_browsing_context(session, closed_frame, mouse_chain):
    with pytest.raises(NoSuchWindowException):
        mouse_chain.click().perform()


def test_click_at_coordinates(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"], duration=1000) \
        .click() \
        .perform()
    events = get_events(session)
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


def test_context_menu_at_coordinates(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .pointer_down(button=2) \
        .pointer_up(button=2) \
        .perform()
    events = get_events(session)
    expected = [
        {"type": "mousedown", "button": 2},
        {"type": "contextmenu", "button": 2},
    ]
    assert len(events) == 4
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_contextmenu_events = [
        x for x in filtered_events
        if x["type"] in ["mousedown", "contextmenu"]
    ]
    assert expected == mousedown_contextmenu_events


def test_click_element_center(session, test_actions_page, mouse_chain):
    outer = session.find.css("#outer", all=False)
    center = get_inview_center(outer.rect, get_viewport_rect(session))
    mouse_chain.click(element=outer).perform()
    events = get_events(session)
    assert len(events) == 4
    event_types = [e["type"] for e in events]
    assert ["mousemove", "mousedown", "mouseup", "click"] == event_types
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == pytest.approx(center["x"], abs=1.0)
            assert e["pageY"] == pytest.approx(center["y"], abs=1.0)
            assert e["target"] == "outer"


def test_click_navigation(session, url, inline):
    destination = url("/webdriver/tests/actions/support/test_actions_wdspec.html")
    start = inline("<a href=\"{}\" id=\"link\">destination</a>".format(destination))

    def click(link):
        mouse_chain = session.actions.sequence(
            "pointer", "pointer_id", {"pointerType": "mouse"})
        mouse_chain.click(element=link).perform()

    session.url = start
    error_message = "Did not navigate to %s" % destination

    click(session.find.css("#link", all=False))
    Poll(session, message=error_message).until(lambda s: s.url == destination)
    # repeat steps to check behaviour after document unload
    session.url = start
    click(session.find.css("#link", all=False))
    Poll(session, message=error_message).until(lambda s: s.url == destination)


def test_pen_pointer_properties(session, test_actions_pointer_page, pen_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    pen_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(pressure=0.36, tilt_x=-72, tilt_y=9, twist=86) \
        .pointer_move(10, 10, origin=pointerArea) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 10
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointermove", "pointerdown",
            "pointerover", "pointerenter", "pointermove", "pointerup",
            "pointerout", "pointerleave"] == event_types
    assert events[3]["type"] == "pointerdown"
    assert events[3]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "pen"
    # The default value of width and height for mouse and pen inputs is 1
    assert round(events[3]["width"], 2) == 1
    assert round(events[3]["height"], 2) == 1
    assert round(events[3]["pressure"], 2) == 0.36
    assert events[3]["tiltX"] == -72
    assert events[3]["tiltY"] == 9
    assert events[3]["twist"] == 86
    assert events[6]["type"] == "pointermove"
    assert events[6]["pageX"] == pytest.approx(center["x"]+10, abs=1.0)
    assert events[6]["pageY"] == pytest.approx(center["y"]+10, abs=1.0)
    assert events[6]["target"] == "pointerArea"
    assert events[6]["pointerType"] == "pen"
    assert round(events[6]["width"], 2) == 1
    assert round(events[6]["height"], 2) == 1
    # The default value of pressure for all inputs is 0.5, other properties are 0
    assert round(events[6]["pressure"], 2) == 0.5
    assert events[6]["tiltX"] == 0
    assert events[6]["tiltY"] == 0
    assert events[6]["twist"] == 0


def test_touch_pointer_properties(session, test_actions_pointer_page, touch_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    touch_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(width=23, height=31, pressure=0.78, tilt_x=21, tilt_y=-8, twist=355) \
        .pointer_move(10, 10, origin=pointerArea, width=39, height=35, pressure=0.91, tilt_x=-19, tilt_y=62, twist=345) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointerdown", "pointermove",
            "pointerup", "pointerout", "pointerleave"] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[2]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[2]["target"] == "pointerArea"
    assert events[2]["pointerType"] == "touch"
    assert round(events[2]["width"], 2) == 23
    assert round(events[2]["height"], 2) == 31
    assert round(events[2]["pressure"], 2) == 0.78
    assert events[2]["tiltX"] == 21
    assert events[2]["tiltY"] == -8
    assert events[2]["twist"] == 355
    assert events[3]["type"] == "pointermove"
    assert events[3]["pageX"] == pytest.approx(center["x"]+10, abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"]+10, abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "touch"
    assert round(events[3]["width"], 2) == 39
    assert round(events[3]["height"], 2) == 35
    assert round(events[3]["pressure"], 2) == 0.91
    assert events[3]["tiltX"] == -19
    assert events[3]["tiltY"] == 62
    assert events[3]["twist"] == 345


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
    initial_center = get_inview_center(initial_rect, get_viewport_rect(session))
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
    # check resulting location of the dragged element
    final_rect = drag_target.rect
    assert initial_rect["x"] + dx == final_rect["x"]
    assert initial_rect["y"] + dy == final_rect["y"]


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
