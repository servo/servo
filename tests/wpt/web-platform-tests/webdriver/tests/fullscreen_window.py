# META: timeout=long

from tests.support.asserts import assert_error, assert_success, assert_dialog_handled
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


alert_doc = inline("<script>window.alert()</script>")


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def fullscreen(session):
    return session.transport.send("POST", "session/%s/window/fullscreen" % session.session_id)


def is_fullscreen(session):
    # At the time of writing, WebKit does not conform to the Fullscreen API specification.
    # Remove the prefixed fallback when https://bugs.webkit.org/show_bug.cgi?id=158125 is fixed.
    return session.execute_script("return !!(window.fullScreen || document.webkitIsFullScreen)")


# 10.7.5 Fullscreen Window


def test_no_browsing_context(session, create_window):
    """
    1. If the current top-level browsing context is no longer open,
    return error with error code no such window.

    """
    session.window_handle = create_window()
    session.close()
    response = fullscreen(session)
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
    create_dialog(session)("alert", text="accept #1", result_var="accept1")

    response = fullscreen(session)

    assert_dialog_handled(session, "accept #1")
    assert read_global(session, "accept1") == None

    expected_title = read_global(session, "document.title")
    create_dialog(session)("confirm", text="accept #2", result_var="accept2")

    response = fullscreen(session)

    assert_dialog_handled(session, "accept #2")
    assert read_global(session, "accept2"), True

    create_dialog(session)("prompt", text="accept #3", result_var="accept3")

    response = fullscreen(session)

    assert_dialog_handled(session, "accept #3")
    assert read_global(session, "accept3") == "" or read_global(session, "accept3") == "undefined"


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

    response = fullscreen(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")
    assert read_global(session, "dismiss1") == None

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = fullscreen(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")
    assert read_global(session, "dismiss2") == False

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = fullscreen(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
    assert read_global(session, "dismiss3") == None


def test_fullscreen(session):
    """
    4. Call fullscreen an element with the current top-level browsing
    context's active document's document element.

    """
    response = fullscreen(session)
    assert_success(response)

    assert is_fullscreen(session) is True


def test_payload(session):
    """
    5. Return success with the JSON serialization of the current top-level
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
    response = fullscreen(session)

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


def test_fullscreen_twice_is_idempotent(session):
    assert is_fullscreen(session) is False

    first_response = fullscreen(session)
    assert_success(first_response)
    assert is_fullscreen(session) is True

    second_response = fullscreen(session)
    assert_success(second_response)
    assert is_fullscreen(session) is True
