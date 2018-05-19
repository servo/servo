from tests.support.asserts import assert_error
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def get_window_rect(session):
    return session.transport.send(
        "GET", "session/{session_id}/window/rect".format(**vars(session)))


def test_no_browsing_context(session, create_window):
    """
    1. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = get_window_rect(session)
    assert_error(response, "no such window")


def test_payload(session):
    """
    3. Return success with the JSON serialization of the current top-level
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
    response = get_window_rect(session)

    assert response.status == 200
    assert isinstance(response.body["value"], dict)
    value = response.body["value"]
    expected = session.execute_script("""return {
         x: window.screenX,
         y: window.screenY,
         width: window.outerWidth,
         height: window.outerHeight
    }""")
    assert expected == value
