def assert_move_to_coordinates(point, target, events):
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == point["x"]
            assert e["pageY"] == point["y"]
            assert e["target"] == target


def get_center(rect):
    return {
        "x": rect["width"] / 2 + rect["x"],
        "y": rect["height"] / 2 + rect["y"],
    }
