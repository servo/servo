from tests.support.asserts import assert_error, assert_dialog_handled, assert_success
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def get_window_rect(session):
    return session.transport.send("GET", "session/%s/window/rect" % session.session_id)


# 10.7.1 Get Window Rect


def test_no_browsing_context(session, create_window):
    """
    1. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = get_window_rect(session)
    assert_error(response, "no such window")


def test_handle_prompt_dismiss_and_notify():
    """TODO"""


def test_handle_prompt_accept_and_notify():
    """TODO"""


def test_handle_prompt_ignore():
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
    session.url = inline("<title>WD doc title</title>")

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")
    response = get_window_rect(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")
    response = get_window_rect(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")
    response = get_window_rect(session)
    assert response.status == 200
    assert_dialog_handled(session, "dismiss #3")


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
    session.url = inline("<title>WD doc title</title>")
    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    response = get_window_rect(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = get_window_rect(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = get_window_rect(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")


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
