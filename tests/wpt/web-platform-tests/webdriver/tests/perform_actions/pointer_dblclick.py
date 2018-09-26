import pytest

from tests.perform_actions.support.refine import filter_dict, get_events
from tests.support.asserts import assert_move_to_coordinates


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
