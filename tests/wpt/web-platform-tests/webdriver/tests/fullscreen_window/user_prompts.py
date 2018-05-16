from tests.support.asserts import assert_error, assert_dialog_handled
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def fullscreen(session):
    return session.transport.send("POST", "session/%s/window/fullscreen" % session.session_id)


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
    _, session = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.url = inline("<title>WD doc title</title>")
    create_dialog(session)("alert", text="accept #1", result_var="accept1")

    fullscreen(session)

    assert_dialog_handled(session, "accept #1")
    assert read_global(session, "accept1") is None

    read_global(session, "document.title")
    create_dialog(session)("confirm", text="accept #2", result_var="accept2")

    fullscreen(session)

    assert_dialog_handled(session, "accept #2")
    assert read_global(session, "accept2"), True

    create_dialog(session)("prompt", text="accept #3", result_var="accept3")

    fullscreen(session)

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
    assert read_global(session, "dismiss1") is None

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = fullscreen(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")
    assert read_global(session, "dismiss2") is False

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = fullscreen(session)

    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
    assert read_global(session, "dismiss3") is None
