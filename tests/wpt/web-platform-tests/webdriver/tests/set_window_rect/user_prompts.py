from tests.support.asserts import assert_dialog_handled, assert_error, assert_success
from tests.support.fixtures import create_dialog


def set_window_rect(session, rect):
    return session.transport.send(
        "POST", "session/{session_id}/window/rect".format(**vars(session)),
        rect)


def test_handle_prompt_dismiss():
    """TODO"""


def test_handle_prompt_accept(new_session, add_browser_capabilites):
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
    original = session.window.rect

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
