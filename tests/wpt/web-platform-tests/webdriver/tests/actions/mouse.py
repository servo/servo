import pytest

from tests.actions.support.mouse import assert_move_to_coordinates, get_center
from tests.actions.support.refine import get_events, filter_dict
from tests.support.inline import inline
from tests.support.wait import wait


def link_doc(dest):
    content = "<a href=\"{}\" id=\"link\">destination</a>".format(dest)
    return inline(content)


# TODO use pytest.approx once we upgrade to pytest > 3.0
def approx(n, m, tolerance=1):
    return abs(n - m) <= tolerance


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
        {"type": "mouseup",  "buttons": 0},
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
        {"type": "contextmenu",  "button": 2},
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
    center = get_center(outer.rect)
    mouse_chain.click(element=outer).perform()
    events = get_events(session)
    assert len(events) == 4
    event_types = [e["type"] for e in events]
    assert ["mousemove", "mousedown", "mouseup", "click"] == event_types
    for e in events:
        if e["type"] != "mousemove":
            assert approx(e["pageX"], center["x"])
            assert approx(e["pageY"], center["y"])
            assert e["target"] == "outer"


def test_click_navigation(session, url, release_actions):
    destination = url("/webdriver/tests/actions/support/test_actions_wdspec.html")
    start = link_doc(destination)

    def click(link):
        mouse_chain = session.actions.sequence(
            "pointer", "pointer_id", {"pointerType": "mouse"})
        mouse_chain.click(element=link).perform()

    session.url = start
    error_message = "Did not navigate to %s" % destination

    click(session.find.css("#link", all=False))
    wait(session, lambda s: s.url == destination, error_message)
    # repeat steps to check behaviour after document unload
    session.url = start
    click(session.find.css("#link", all=False))
    wait(session, lambda s: s.url == destination, error_message)


@pytest.mark.parametrize("drag_duration", [0, 300, 800])
@pytest.mark.parametrize("dx, dy",
    [(20, 0), (0, 15), (10, 15), (-20, 0), (10, -15), (-10, -15)])
def test_drag_and_drop(session,
                       test_actions_page,
                       mouse_chain,
                       dx,
                       dy,
                       drag_duration):
    drag_target = session.find.css("#dragTarget", all=False)
    initial_rect = drag_target.rect
    initial_center = get_center(initial_rect)
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
    assert approx(e["pageX"], initial_center["x"] + dx)
    assert approx(e["pageY"], initial_center["y"] + dy)
    # check resulting location of the dragged element
    final_rect = drag_target.rect
    assert initial_rect["x"] + dx == final_rect["x"]
    assert initial_rect["y"] + dy == final_rect["y"]
