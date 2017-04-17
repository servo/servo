from support.refine import get_events, filter_dict


def test_click_at_coordinates(session, test_actions_page, mouse_chain):
    div_point = {
        "x": 82,
        "y": 187,
    }
    button = 0
    mouse_chain \
        .pointer_move(div_point["x"], div_point["y"], duration=1000) \
        .pointer_down(button) \
        .pointer_up(button) \
        .perform()
    events = get_events(session)
    assert len(events) == 4
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == div_point["x"]
            assert e["pageY"] == div_point["y"]
            assert e["target"] == "outer"
        if e["type"] != "mousedown":
            assert e["buttons"] == 0
        assert e["button"] == button
    expected = [
        {"type": "mousedown", "buttons": 1},
        {"type": "mouseup",  "buttons": 0},
        {"type": "click", "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]
