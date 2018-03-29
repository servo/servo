# META: timeout=long

from tests.support.asserts import assert_error, assert_dialog_handled, assert_success
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def maximize(session):
    return session.transport.send("POST", "session/%s/window/maximize" % session.session_id)


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the Fullscreen API specification.
    # Remove the prefixed fallback when https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("return !!(window.fullScreen || document.webkitIsFullScreen)")


# 10.7.3 Maximize Window


def test_no_browsing_context(session, create_window):
    """
    2. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = maximize(session)
    assert_error(response, "no such window")


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
    """TODO"""


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    """
    3. Handle any user prompts and return its value if it is an error.

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
    session.url = inline("<title>WD doc title</title>")

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")
    response = maximize(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")
    response = maximize(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")
    response = maximize(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_missing_value(session, create_dialog):
    """
    3. Handle any user prompts and return its value if it is an error.

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
    session.url = inline("<title>WD doc title</title>")
    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    response = maximize(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = maximize(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = maximize(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")


def test_fully_exit_fullscreen(session):
    """
    4. Fully exit fullscreen.

    [...]

    To fully exit fullscreen a document document, run these steps:

      1. If document's fullscreen element is null, terminate these steps.

      2. Unfullscreen elements whose fullscreen flag is set, within
      document's top layer, except for document's fullscreen element.

      3. Exit fullscreen document.

    """
    session.window.fullscreen()
    assert is_fullscreen(session) is True

    response = maximize(session)
    assert_success(response)
    assert is_fullscreen(session) is False


def test_restore_the_window(session):
    """
    5. Restore the window.

    [...]

    To restore the window, given an operating system level window with
    an associated top-level browsing context, run implementation-specific
    steps to restore or unhide the window to the visible screen.  Do not
    return from this operation until the visibility state of the top-level
    browsing context's active document has reached the visible state,
    or until the operation times out.

    """
    session.window.minimize()
    assert session.execute_script("return document.hidden") is True

    response = maximize(session)
    assert_success(response)


def test_maximize(session):
    """
    6. Maximize the window of the current browsing context.

    [...]

    To maximize the window, given an operating system level window with an
    associated top-level browsing context, run the implementation-specific
    steps to transition the operating system level window into the
    maximized window state.  If the window manager supports window
    resizing but does not have a concept of window maximation, the window
    dimensions must be increased to the maximum available size permitted
    by the window manager for the current screen.  Return when the window
    has completed the transition, or within an implementation-defined
    timeout.

    """
    before_size = session.window.size

    response = maximize(session)
    assert_success(response)

    assert before_size != session.window.size


def test_payload(session):
    """
    7. Return success with the JSON serialization of the current top-level
    browsing context's window rect.

    [...]

    A top-level browsing context's window rect is defined as a
    dictionary of the screenX, screenY, width and height attributes of
    the WindowProxy. Its JSON representation is the following:

    "x"
        WindowProxy's screenX attribute.

    "y"
        WindowProxy's screenY attribute.

    "width"
        Width of the top-level browsing context's outer dimensions,
        including any browser chrome and externally drawn window
        decorations in CSS reference pixels.

    "height"
        Height of the top-level browsing context's outer dimensions,
        including any browser chrome and externally drawn window
        decorations in CSS reference pixels.

    """
    before_size = session.window.size

    response = maximize(session)

    # step 5
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

    assert before_size != session.window.size


def test_maximize_twice_is_idempotent(session):
    first_response = maximize(session)
    assert_success(first_response)
    max_size = session.window.size

    second_response = maximize(session)
    assert_success(second_response)
    assert session.window.size == max_size


"""
TODO(ato): Implicit session start does not use configuration passed on
from wptrunner.  This causes an exception.

See https://bugzil.la/1398459.

def test_maximize_when_resized_to_max_size(session):
    # Determine the largest available window size by first maximising
    # the window and getting the window rect dimensions.
    #
    # Then resize the window to the maximum available size.
    session.end()
    available = session.window.maximize()
    session.end()

    session.window.size = available

    # In certain window managers a window extending to the full available
    # dimensions of the screen may not imply that the window is maximised,
    # since this is often a special state.  If a remote end expects a DOM
    # resize event, this may not fire if the window has already reached
    # its expected dimensions.
    before = session.window.size
    session.window.maximize()
    assert session.window.size == before
"""
