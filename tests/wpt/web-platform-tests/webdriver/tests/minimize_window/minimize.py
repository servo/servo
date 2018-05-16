from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def minimize(session):
    return session.transport.send(
        "POST", "session/{session_id}/window/minimize".format(**vars(session)))


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the Fullscreen API specification.
    # Remove the prefixed fallback when https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("return !!(window.fullScreen || document.webkitIsFullScreen)")

# 10.7.4 Minimize Window


def test_no_browsing_context(session, create_window):
    """
    1. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = minimize(session)
    assert_error(response, "no such window")


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

    response = minimize(session)
    assert_success(response)
    assert is_fullscreen(session) is False
    assert session.execute_script("return document.hidden") is True


def test_minimize(session):
    """
    5. Iconify the window.

    [...]

    To iconify the window, given an operating system level window with an
    associated top-level browsing context, run implementation-specific
    steps to iconify, minimize, or hide the window from the visible
    screen. Do not return from this operation until the visibility state
    of the top-level browsing context's active document has reached the
    hidden state, or until the operation times out.

    """
    assert not session.execute_script("return document.hidden")

    response = minimize(session)
    assert_success(response)

    assert session.execute_script("return document.hidden")


def test_payload(session):
    """
    6. Return success with the JSON serialization of the current top-level
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
    assert not session.execute_script("return document.hidden")

    response = minimize(session)

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

    assert session.execute_script("return document.hidden")


def test_minimize_twice_is_idempotent(session):
    assert not session.execute_script("return document.hidden")

    first_response = minimize(session)
    assert_success(first_response)
    assert session.execute_script("return document.hidden")

    second_response = minimize(session)
    assert_success(second_response)
    assert session.execute_script("return document.hidden")
