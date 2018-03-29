# META: timeout=long

import pytest

from tests.support.asserts import assert_error, assert_dialog_handled, assert_success
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def set_window_rect(session, rect):
    return session.transport.send("POST", "session/%s/window/rect" % session.session_id, rect)

def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the Fullscreen API specification.
    # Remove the prefixed fallback when https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("return !!(window.fullScreen || document.webkitIsFullScreen)")


# 10.7.2 Set Window Rect


def test_current_top_level_browsing_context_no_longer_open(session, create_window):
    """
    1. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = set_window_rect(session, {})
    assert_error(response, "no such window")


def test_handle_prompt_dismiss():
    """TODO"""


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    """
    2. Handle any user prompts and return its value if it is an error.

    [...]

    In order to handle any user prompts a remote end must take the
    following steps:

      [...]

      2. Perform the following substeps based on the current session's
      user prompt handler:

        [...]

        - accept state
           Accept the current user prompt.

    """
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    original = session.window.rect

    # step 2
    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")
    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert result.status == 200
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")
    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert result.status == 200
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")
    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert_success(result)
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


def test_handle_prompt_missing_value(session, create_dialog):
    """
    2. Handle any user prompts and return its value if it is an error.

    [...]

    In order to handle any user prompts a remote end must take the
    following steps:

      [...]

      2. Perform the following substeps based on the current session's
      user prompt handler:

        [...]

        - missing value default state
           1. Dismiss the current user prompt.
           2. Return error with error code unexpected alert open.

    """

    original = session.window.rect

    # step 2
    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    result = set_window_rect(session, {"x": original["x"],
                                       "y": original["y"]})
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")


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
    """
    8. If width or height is neither null nor a Number from 0 to 2^31 -
    1, return error with error code invalid argument.

    9. If x or y is neither null nor a Number from -(2^31) to 2^31 - 1,
    return error with error code invalid argument.
    """
    response = set_window_rect(session, rect)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("rect", [
    {"width": -1},
    {"height": -2},
    {"width": -1, "height": -2},
])
def test_out_of_bounds(session, rect):
    """
    8. If width or height is neither null nor a Number from 0 to 2^31 -
    1, return error with error code invalid argument.

    9. If x or y is neither null nor a Number from -(2^31) to 2^31 - 1,
    return error with error code invalid argument.
    """
    response = set_window_rect(session, rect)
    assert_error(response, "invalid argument")


def test_width_height_floats(session):
    """
    8. If width or height is neither null nor a Number from 0 to 2^31 -
    1, return error with error code invalid argument.
    """

    response = set_window_rect(session, {"width": 500.5, "height": 420})
    value = assert_success(response)
    assert value["width"] == 500
    assert value["height"] == 420

    response = set_window_rect(session, {"width": 500, "height": 450.5})
    value = assert_success(response)
    assert value["width"] == 500
    assert value["height"] == 450


def test_x_y_floats(session):
    """
    9. If x or y is neither null nor a Number from -(2^31) to 2^31 - 1,
    return error with error code invalid argument.
    """

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
    """
    13. If width and height are not null:

    [...]

    14. If x and y are not null:

    [...]

    15. Return success with the JSON serialization of the current
    top-level browsing context's window rect.
    """

    original = session.window.rect
    response = set_window_rect(session, rect)
    assert_success(response, original)


def test_fully_exit_fullscreen(session):
    """
    10. Fully exit fullscreen.

    [...]

    To fully exit fullscreen a document document, run these steps:

      1. If document's fullscreen element is null, terminate these steps.

      2. Unfullscreen elements whose fullscreen flag is set, within
      document's top layer, except for document's fullscreen element.

      3. Exit fullscreen document.
    """
    session.window.fullscreen()
    assert is_fullscreen(session) is True

    response = set_window_rect(session, {"width": 400, "height": 400})
    value = assert_success(response)
    assert value["width"] == 400
    assert value["height"] == 400

    assert is_fullscreen(session) is False


def test_restore_from_minimized(session):
    """
    12. If the visibility state of the top-level browsing context's
    active document is hidden, restore the window.

    [...]

    To restore the window, given an operating system level window with
    an associated top-level browsing context, run implementation-specific
    steps to restore or unhide the window to the visible screen. Do not
    return from this operation until the visibility state of the top-level
    browsing context's active document has reached the visible state,
    or until the operation times out.
    """

    session.window.minimize()
    assert session.execute_script("return document.hidden") is True

    response = set_window_rect(session, {"width": 450, "height": 450})
    value = assert_success(response)
    assert value["width"] == 450
    assert value["height"] == 450

    assert session.execute_script("return document.hidden") is False


def test_restore_from_maximized(session):
    """
    12. If the visibility state of the top-level browsing context's
    active document is hidden, restore the window.

    [...]

    To restore the window, given an operating system level window with
    an associated top-level browsing context, run implementation-specific
    steps to restore or unhide the window to the visible screen. Do not
    return from this operation until the visibility state of the top-level
    browsing context's active document has reached the visible state,
    or until the operation times out.
    """

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

    # step 12
    response = set_window_rect(session, {"width": max["width"] - 100,
                                         "height": max["height"] - 100})

    # step 14
    assert_success(response, {"x": original["x"],
                              "y": original["y"],
                              "width": max["width"] - 100,
                              "height": max["height"] - 100})


def test_height_width_larger_than_max(session):
    max = session.execute_script("""
        return {
          width: window.screen.availWidth,
          height: window.screen.availHeight,
        }""")

    # step 12
    response = set_window_rect(session, {"width": max["width"] + 100,
                                         "height": max["height"] + 100})

    # step 14
    rect = assert_success(response)
    assert rect["width"] >= max["width"]
    assert rect["height"] >= max["height"]


def test_height_width_as_current(session):
    original = session.window.rect

    # step 12
    response = set_window_rect(session, {"width": original["width"],
                                         "height": original["height"]})

    # step 14
    assert_success(response, {"x": original["x"],
                              "y": original["y"],
                              "width": original["width"],
                              "height": original["height"]})


def test_x_y(session):
    original = session.window.rect

    # step 13
    response = set_window_rect(session, {"x": original["x"] + 10,
                                         "y": original["y"] + 10})

    # step 14
    assert_success(response, {"x": original["x"] + 10,
                              "y": original["y"] + 10,
                              "width": original["width"],
                              "height": original["height"]})


def test_negative_x_y(session):
    original = session.window.rect

    # step 13
    response = set_window_rect(session, {"x": - 8, "y": - 8})

    # step 14
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
    elif os == "darwin":
        assert_success(response, {"x": -8,
                                  "y": 23,
                                  "width": original["width"],
                                  "height": original["height"]})

    # It turns out that Windows is the only platform on which the
    # window can be reliably positioned off-screen.
    elif os == "windows_nt":
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


def test_payload(session):
    # step 14
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
