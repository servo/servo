# META: timeout=long

import pytest

from webdriver.error import NoSuchWindowException

from tests.perform_actions.support.mouse import get_inview_center, get_viewport_rect
from tests.perform_actions.support.refine import filter_dict, get_events
from tests.support.asserts import assert_move_to_coordinates
from tests.support.inline import inline
from tests.support.sync import Poll


def link_doc(dest):
    content = "<a href=\"{}\" id=\"link\">destination</a>".format(dest)
    return inline(content)


def test_null_response_value(session, mouse_chain):
    value = mouse_chain.click().perform()
    assert value is None


def test_no_browsing_context(session, closed_window, mouse_chain):
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
            assert e["pageX"] == pytest.approx(center["x"], abs = 1.0)
            assert e["pageY"] == pytest.approx(center["y"], abs = 1.0)
            assert e["target"] == "outer"


def test_click_navigation(session, url):
    destination = url("/webdriver/tests/actions/support/test_actions_wdspec.html")
    start = link_doc(destination)

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
    assert e["pageX"] == pytest.approx(initial_center["x"] + dx, abs = 1.0)
    assert e["pageY"] == pytest.approx(initial_center["y"] + dy, abs = 1.0)
    # check resulting location of the dragged element
    final_rect = drag_target.rect
    assert initial_rect["x"] + dx == final_rect["x"]
    assert initial_rect["y"] + dy == final_rect["y"]
