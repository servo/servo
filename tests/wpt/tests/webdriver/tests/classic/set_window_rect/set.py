# META: timeout=long

import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import document_hidden, is_fullscreen, is_maximized


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/window/rect".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_top_browsing_context(session, closed_window):
    response = set_window_rect(session, {})
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_window):
    response = set_window_rect(session, {})
    assert_error(response, "no such window")


def test_response_payload(session):
    response = set_window_rect(session, {"x": 400, "y": 400})
    value = assert_success(response, session.window.rect)

    assert isinstance(value, dict)
    assert isinstance(value.get("x"), int)
    assert isinstance(value.get("y"), int)
    assert isinstance(value.get("width"), int)
    assert isinstance(value.get("height"), int)


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
def test_invalid_values(session, rect):
    response = set_window_rect(session, rect)
    assert_error(response, "invalid argument")


def test_restore_from_fullscreen(session):
    assert not is_fullscreen(session)

    original = session.window.rect
    target_rect = {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 50,
        "height": original["height"] + 50
    }

    session.window.fullscreen()
    assert is_fullscreen(session)

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert not is_fullscreen(session)
    assert value == target_rect


def test_restore_from_minimized(session):
    assert not document_hidden(session)

    original = session.window.rect
    target_rect = {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 50,
        "height": original["height"] + 50
    }

    session.window.minimize()
    assert document_hidden(session)

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert not document_hidden(session)
    assert value == target_rect


def test_restore_from_maximized(session):
    assert not is_maximized(session)

    original = session.window.rect
    target_rect = {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 50,
        "height": original["height"] + 50
    }

    session.window.maximize()
    assert is_maximized(session)

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert not is_maximized(session)
    assert value == target_rect


def test_x_y_floats(session):
    response = set_window_rect(session, {"x": 150.5, "y": 250})
    value = assert_success(response)
    assert value["x"] == 150
    assert value["y"] == 250

    response = set_window_rect(session, {"x": 150, "y": 250.5})
    value = assert_success(response, session.window.rect)
    assert value["x"] == 150
    assert value["y"] == 250


def test_width_height_floats(session):
    response = set_window_rect(session, {"width": 500.5, "height": 420})
    value = assert_success(response, session.window.rect)
    assert value["width"] == 500
    assert value["height"] == 420

    response = set_window_rect(session, {"width": 500, "height": 450.5})
    value = assert_success(response, session.window.rect)
    assert value["width"] == 500
    assert value["height"] == 450


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


def test_set_to_available_size(
    session, available_screen_size, minimal_screen_position
):
    minimal_x, minimal_y = minimal_screen_position
    available_width, available_height = available_screen_size
    target_rect = {
        "x": minimal_x,
        "y": minimal_y,
        "width": available_width,
        "height": available_height,
    }

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert value == target_rect


def test_set_to_screen_size(
    session, available_screen_size, minimal_screen_position, screen_size
):
    minimal_x, minimal_y = minimal_screen_position
    available_width, available_height = available_screen_size
    screen_width, screen_height = screen_size
    target_rect = {
        "x": minimal_x,
        "y": minimal_y,
        "width": screen_width,
        "height": screen_height,
    }

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert value["width"] >= available_width
    assert value["width"] <= screen_width
    assert value["height"] >= available_height
    assert value["height"] <= screen_height


def test_set_larger_than_screen_size(
    session, available_screen_size, minimal_screen_position, screen_size
):
    minimal_x, minimal_y = minimal_screen_position
    available_width, available_height = available_screen_size
    screen_width, screen_height = screen_size
    target_rect = {
        "x": minimal_x,
        "y": minimal_y,
        "width": screen_width + 100,
        "height": screen_height + 100,
    }

    response = set_window_rect(session, target_rect)
    value = assert_success(response, session.window.rect)

    assert value["width"] >= available_width
    assert value["height"] >= available_height


def test_set_smaller_than_minimum_browser_size(session):
    original_width, original_height = session.window.size

    # A window size of 10x10px shouldn't be supported by any browser.
    response = set_window_rect(session, {"width": 10, "height": 10})
    value = assert_success(response, session.window.rect)

    assert value["width"] < original_width
    assert value["width"] > 10
    assert value["height"] < original_height
    assert value["height"] > 10


def test_height_width_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"],
        "height": original["height"]
    })
    value = assert_success(response, session.window.rect)

    assert value == original


def test_height_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"] + 10,
        "height": original["height"]
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 10,
        "height": original["height"]
    }


def test_width_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"],
        "height": original["height"] + 10
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"],
        "height": original["height"] + 10
    }


def test_x_y(session):
    original = session.window.rect
    response = set_window_rect(session, {
        "x": original["x"] + 10,
        "y": original["y"] + 10
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"] + 10,
        "y": original["y"] + 10,
        "width": original["width"],
        "height": original["height"]
    }


def test_x_y_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"],
        "y": original["y"]
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"],
        "height": original["height"]
    }


def test_x_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"],
        "y": original["y"] + 10
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"],
        "y": original["y"] + 10,
        "width": original["width"],
        "height": original["height"]
    }


def test_y_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"] + 10,
        "y": original["y"]
    })
    value = assert_success(response, session.window.rect)

    assert value == {
        "x": original["x"] + 10,
        "y": original["y"],
        "width": original["width"],
        "height": original["height"]
    }


def test_negative_x_y(session, minimal_screen_position):
    original = session.window.rect

    response = set_window_rect(session, {"x": - 8, "y": - 8})
    value = assert_success(response, session.window.rect)

    os = session.capabilities["platformName"]
    # certain WMs prohibit windows from being moved off-screen
    if os == "linux":
        assert value["x"] <= 0
        assert value["y"] <= 0
        assert value["width"] == original["width"]
        assert value["height"] == original["height"]

    # On macOS when not running headless, windows can only be moved off the
    # screen on the horizontal axis.  The system menu bar also blocks windows
    # from being moved to (0,0).
    elif os == "mac":
        assert value["x"] == -8
        assert value["y"] <= minimal_screen_position[1]
        assert value["width"] == original["width"]
        assert value["height"] == original["height"]

    # It turns out that Windows is the only platform on which the
    # window can be reliably positioned off-screen.
    elif os == "windows":
        assert value == {
            "x": -8,
            "y": -8,
            "width": original["width"],
            "height": original["height"]
        }


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
