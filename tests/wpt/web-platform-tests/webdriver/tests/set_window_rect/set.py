# META: timeout=long

import pytest

from tests.support.asserts import assert_error, assert_success


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the
    # Fullscreen API specification.
    #
    # Remove the prefixed fallback when
    # https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("""
        return !!(window.fullScreen || document.webkitIsFullScreen)
        """)


def test_current_top_level_browsing_context_no_longer_open(session, create_window):
    session.window_handle = create_window()
    session.close()
    response = set_window_rect(session, {})
    assert_error(response, "no such window")


@pytest.mark.parametrize("rect", [
    {"width": "a"},
    {"height": "b"},
    {"width": "a", "height": "b"},
    {"x": "a"},
    {"y": "b"},
    {"x": "a", "y": "b"},
    {"width": "a", "height": "b", "x": "a", "y": "b"},

    {"width": True},
    {"height": False},
    {"width": True, "height": False},
    {"x": True},
    {"y": False},
    {"x": True, "y": False},
    {"width": True, "height": False, "x": True, "y": False},

    {"width": []},
    {"height": []},
    {"width": [], "height": []},
    {"x": []},
    {"y": []},
    {"x": [], "y": []},
    {"width": [], "height": [], "x": [], "y": []},

    {"height": {}},
    {"width": {}},
    {"height": {}, "width": {}},
    {"x": {}},
    {"y": {}},
    {"x": {}, "y": {}},
    {"width": {}, "height": {}, "x": {}, "y": {}},
])
def test_invalid_types(session, rect):
    response = set_window_rect(session, rect)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("rect", [
    {"width": -1},
    {"height": -2},
    {"width": -1, "height": -2},
])
def test_out_of_bounds(session, rect):
    response = set_window_rect(session, rect)
    assert_error(response, "invalid argument")


def test_width_height_floats(session):
    response = set_window_rect(session, {"width": 500.5, "height": 420})
    value = assert_success(response)
    assert value["width"] == 500
    assert value["height"] == 420

    response = set_window_rect(session, {"width": 500, "height": 450.5})
    value = assert_success(response)
    assert value["width"] == 500
    assert value["height"] == 450


def test_x_y_floats(session):
    response = set_window_rect(session, {"x": 0.5, "y": 420})
    value = assert_success(response)
    assert value["x"] == 0
    assert value["y"] == 420

    response = set_window_rect(session, {"x": 100, "y": 450.5})
    value = assert_success(response)
    assert value["x"] == 100
    assert value["y"] == 450


@pytest.mark.parametrize("rect", [
    {},

    {"width": None},
    {"height": None},
    {"width": None, "height": None},

    {"x": None},
    {"y": None},
    {"x": None, "y": None},

    {"width": None, "x": None},
    {"width": None, "y": None},
    {"height": None, "x": None},
    {"height": None, "Y": None},

    {"width": None, "height": None, "x": None, "y": None},

    {"width": 200},
    {"height": 200},
    {"x": 200},
    {"y": 200},
    {"width": 200, "x": 200},
    {"height": 200, "x": 200},
    {"width": 200, "y": 200},
    {"height": 200, "y": 200},
])
def test_no_change(session, rect):
    original = session.window.rect
    response = set_window_rect(session, rect)
    assert_success(response, original)


def test_fully_exit_fullscreen(session):
    session.window.fullscreen()
    assert is_fullscreen(session) is True

    response = set_window_rect(session, {"width": 400, "height": 400})
    value = assert_success(response)
    assert value["width"] == 400
    assert value["height"] == 400

    assert is_fullscreen(session) is False


def test_restore_from_minimized(session):
    session.window.minimize()
    assert session.execute_script("return document.hidden") is True

    response = set_window_rect(session, {"width": 450, "height": 450})
    value = assert_success(response)
    assert value["width"] == 450
    assert value["height"] == 450

    assert session.execute_script("return document.hidden") is False


def test_restore_from_maximized(session):
    original_size = session.window.size
    session.window.maximize()
    assert session.window.size != original_size

    response = set_window_rect(session, {"width": 400, "height": 400})
    value = assert_success(response)
    assert value["width"] == 400
    assert value["height"] == 400


def test_height_width(session):
    original = session.window.rect
    max = session.execute_script("""
        return {
          width: window.screen.availWidth,
          height: window.screen.availHeight,
        }""")

    response = set_window_rect(session, {
        "width": max["width"] - 100,
        "height": max["height"] - 100
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": max["width"] - 100,
        "height": max["height"] - 100
    })


def test_height_width_larger_than_max(session):
    max = session.execute_script("""
        return {
          width: window.screen.availWidth,
          height: window.screen.availHeight,
        }""")

    response = set_window_rect(session, {
        "width": max["width"] + 100,
        "height": max["height"] + 100
    })
    rect = assert_success(response)
    assert rect["width"] >= max["width"]
    assert rect["height"] >= max["height"]


def test_height_width_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"],
        "height": original["height"]
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"],
        "height": original["height"]
    })


def test_x_y(session):
    original = session.window.rect
    response = set_window_rect(session, {
        "x": original["x"] + 10,
        "y": original["y"] + 10
    })
    assert_success(response, {
        "x": original["x"] + 10,
        "y": original["y"] + 10,
        "width": original["width"],
        "height": original["height"]
    })


def test_negative_x_y(session):
    original = session.window.rect

    response = set_window_rect(session, {"x": - 8, "y": - 8})

    os = session.capabilities["platformName"]
    # certain WMs prohibit windows from being moved off-screen
    if os == "linux":
        rect = assert_success(response)
        assert rect["x"] <= 0
        assert rect["y"] <= 0
        assert rect["width"] == original["width"]
        assert rect["height"] == original["height"]

    # On macOS, windows can only be moved off the screen on the
    # horizontal axis.  The system menu bar also blocks windows from
    # being moved to (0,0).
    elif os == "mac":
        assert_success(response, {"x": -8,
                                  "y": 23,
                                  "width": original["width"],
                                  "height": original["height"]})

    # It turns out that Windows is the only platform on which the
    # window can be reliably positioned off-screen.
    elif os == "windows":
        assert_success(response, {"x": -8,
                                  "y": -8,
                                  "width": original["width"],
                                  "height": original["height"]})


def test_move_to_same_position(session):
    original_position = session.window.position
    position = session.window.position = original_position
    assert position == original_position


def test_move_to_same_x(session):
    original_x = session.window.position[0]
    position = session.window.position = (original_x, 345)
    assert position == (original_x, 345)


def test_move_to_same_y(session):
    original_y = session.window.position[1]
    position = session.window.position = (456, original_y)
    assert position == (456, original_y)


def test_resize_to_same_size(session):
    original_size = session.window.size
    size = session.window.size = original_size
    assert size == original_size


def test_resize_to_same_width(session):
    original_width = session.window.size[0]
    size = session.window.size = (original_width, 345)
    assert size == (original_width, 345)


def test_resize_to_same_height(session):
    original_height = session.window.size[1]
    size = session.window.size = (456, original_height)
    assert size == (456, original_height)


"""
TODO(ato):

    Disable test because the while statements are wrong.
    To fix this properly we need to write an explicit wait utility.

def test_resize_by_script(session):
    # setting the window size by JS is asynchronous
    # so we poll waiting for the results

    size0 = session.window.size

    session.execute_script("window.resizeTo(700, 800)")
    size1 = session.window.size
    while size0 == size1:
        size1 = session.window.size
    assert size1 == (700, 800)

    session.execute_script("window.resizeTo(800, 900)")
    size2 = session.window.size
    while size1 == size2:
        size2 = session.window.size
        assert size2 == (800, 900)
    assert size2 == {"width": 200, "height": 100}
"""


def test_payload(session):
    response = set_window_rect(session, {"x": 400, "y": 400})

    assert response.status == 200
    assert isinstance(response.body["value"], dict)
    value = response.body["value"]
    assert "width" in value
    assert "height" in value
    assert "x" in value
    assert "y" in value
    assert isinstance(value["width"], int)
    assert isinstance(value["height"], int)
    assert isinstance(value["x"], int)
    assert isinstance(value["y"], int)
