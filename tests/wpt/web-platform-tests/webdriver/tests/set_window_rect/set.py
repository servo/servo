# META: timeout=long

import pytest

from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.helpers import (available_screen_size, document_hidden,
                                   is_fullscreen, screen_size)


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/window/rect".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_no_browsing_context(session, closed_window):
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
    response = set_window_rect(session, {"width": 750.5, "height": 700})
    value = assert_success(response)
    assert value["width"] == 750
    assert value["height"] == 700

    response = set_window_rect(session, {"width": 750, "height": 700.5})
    value = assert_success(response)
    assert value["width"] == 750
    assert value["height"] == 700


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
    assert is_fullscreen(session)

    response = set_window_rect(session, {"width": 600, "height": 400})
    value = assert_success(response)
    assert value["width"] == 600
    assert value["height"] == 400

    assert not is_fullscreen(session)


def test_restore_from_minimized(session):
    session.window.minimize()
    assert document_hidden(session)

    response = set_window_rect(session, {"width": 750, "height": 700})
    value = assert_success(response)
    assert value["width"] == 750
    assert value["height"] == 700

    assert not document_hidden(session)


def test_restore_from_maximized(session):
    original_size = session.window.size
    session.window.maximize()
    assert session.window.size != original_size

    response = set_window_rect(session, {"width": 750, "height": 700})
    value = assert_success(response)
    assert value["width"] == 750
    assert value["height"] == 700


def test_height_width(session):
    # The window position might be auto-adjusted by the browser
    # if it exceeds the lower right corner. As such ensure that
    # there is enough space left so no window move will occur.
    session.window.position = (50, 50)

    original = session.window.rect
    screen_width, screen_height = screen_size(session)

    response = set_window_rect(session, {
        "width": screen_width - 100,
        "height": screen_height - 100
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": screen_width - 100,
        "height": screen_height - 100,
    })


def test_height_width_smaller_than_minimum_browser_size(session):
    original = session.window.rect

    response = set_window_rect(session, {"width": 10, "height": 10})
    rect = assert_success(response)
    assert rect["width"] < original["width"]
    assert rect["width"] > 10
    assert rect["height"] < original["height"]
    assert rect["height"] > 10


def test_height_width_larger_than_max(session):
    screen_width, screen_height = screen_size(session)
    avail_width, avail_height = available_screen_size(session)

    response = set_window_rect(session, {
        "width": screen_width + 100,
        "height": screen_height + 100
    })
    rect = assert_success(response)
    assert rect["width"] >= avail_width
    assert rect["height"] >= avail_height


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


def test_height_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"] + 10,
        "height": original["height"]
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"] + 10,
        "height": original["height"]
    })


def test_width_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "width": original["width"],
        "height": original["height"] + 10
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"],
        "height": original["height"] + 10
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
        value = assert_success(response)

        # `screen.availTop` is not standardized but all browsers we care
        # about on MacOS implement the CSSOM View mode `Screen` interface.
        avail_top = session.execute_script("return window.screen.availTop;")

        assert value == {"x": -8,
                         "y": avail_top,
                         "width": original["width"],
                         "height": original["height"]}

    # It turns out that Windows is the only platform on which the
    # window can be reliably positioned off-screen.
    elif os == "windows":
        assert_success(response, {"x": -8,
                                  "y": -8,
                                  "width": original["width"],
                                  "height": original["height"]})


def test_x_y_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"],
        "y": original["y"]
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"],
        "width": original["width"],
        "height": original["height"]
    })


def test_x_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"],
        "y": original["y"] + 10
    })
    assert_success(response, {
        "x": original["x"],
        "y": original["y"] + 10,
        "width": original["width"],
        "height": original["height"]
    })


def test_y_as_current(session):
    original = session.window.rect

    response = set_window_rect(session, {
        "x": original["x"] + 10,
        "y": original["y"]
    })
    assert_success(response, {
        "x": original["x"] + 10,
        "y": original["y"],
        "width": original["width"],
        "height": original["height"]
    })


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
