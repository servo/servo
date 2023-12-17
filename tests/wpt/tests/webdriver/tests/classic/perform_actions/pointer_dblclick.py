import pytest

from tests.classic.perform_actions.support.refine import get_events
from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict


@pytest.mark.parametrize("click_pause", [0, 200])
def test_dblclick_at_coordinates(session, test_actions_page, mouse_chain, click_pause):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .click() \
        .pause(click_pause) \
        .click() \
        .perform()
    events = get_events(session)
    assert_move_to_coordinates(div_point, "outer", events)
    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "dblclick", "button": 0},
    ]
    assert len(events) == 8
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


def test_no_dblclick_when_mouse_moves(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"]) \
        .click() \
        .pointer_move(div_point["x"] + 10, div_point["y"] + 10) \
        .click() \
        .perform()
    events = get_events(session)
    expected = [
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
        {"type": "mousedown", "button": 0},
        {"type": "mouseup", "button": 0},
        {"type": "click", "button": 0},
    ]
    assert len(events) == 7
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]
